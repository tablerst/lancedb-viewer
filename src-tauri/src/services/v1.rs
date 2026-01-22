use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use arrow_array::{RecordBatch, RecordBatchIterator};
use arrow_csv::{ReaderBuilder as CsvReaderBuilder, WriterBuilder as CsvWriterBuilder};
use arrow_ipc::writer::StreamWriter;
use arrow_json::{ArrayWriter, ReaderBuilder};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use base64::{engine::general_purpose, Engine as _};
use futures_util::TryStreamExt;
use lancedb::index::scalar::{
    BTreeIndexBuilder, BitmapIndexBuilder, FtsIndexBuilder, FullTextSearchQuery,
    LabelListIndexBuilder,
};
use lancedb::index::vector::{
    IvfFlatIndexBuilder, IvfHnswPqIndexBuilder, IvfHnswSqIndexBuilder, IvfPqIndexBuilder,
    IvfRqIndexBuilder, IvfSqIndexBuilder,
};
use lancedb::index::{Index, IndexType};
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use lancedb::table::{
    AddDataMode, ColumnAlteration, CompactionOptions, Duration as LanceDuration, NewColumnTransform,
    OptimizeAction,
};
use lancedb::Table;
use log::{debug, error, info, trace, warn};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;

use crate::domain::connect::infer_backend_kind;
use crate::ipc::v1::{
    AddColumnsRequestV1, AddColumnsResponseV1, AlterColumnsRequestV1, AlterColumnsResponseV1,
    ArrowChunk, CheckoutTableLatestRequestV1, CheckoutTableLatestResponseV1,
    CheckoutTableVersionRequestV1, CheckoutTableVersionResponseV1, CloneTableRequestV1,
    CloneTableResponseV1, ColumnAlterationInput, CombinedSearchRequestV1, ConnectRequestV1,
    ConnectResponseV1, CreateIndexRequestV1, CreateIndexResponseV1, CreateTableRequestV1,
    CreateTableResponseV1, DataChunk, DataFileFormatV1, DataFormat, DeleteRowsRequestV1,
    DeleteRowsResponseV1, DisconnectRequestV1, DisconnectResponseV1, DropColumnsRequestV1,
    DropColumnsResponseV1, DropIndexRequestV1, DropIndexResponseV1, DropTableRequestV1,
    DropTableResponseV1, ErrorCode, ExportDataRequestV1, ExportDataResponseV1, FieldDataType,
    FtsSearchRequestV1, GetSchemaRequestV1, GetTableVersionRequestV1, GetTableVersionResponseV1,
    ImportDataRequestV1, ImportDataResponseV1, IndexDefinitionV1, IndexTypeV1, JsonChunk,
    ListIndexesRequestV1, ListIndexesResponseV1, ListTablesRequestV1, ListTablesResponseV1,
    ListVersionsRequestV1, ListVersionsResponseV1, OpenTableRequestV1, OptimizeActionV1,
    OptimizeTableRequestV1, OptimizeTableResponseV1, QueryFilterRequestV1, QueryResponseV1,
    RenameTableRequestV1, RenameTableResponseV1, ResultEnvelope, ScanRequestV1, ScanResponseV1,
    SchemaDefinition, SchemaDefinitionInput, SchemaFieldInput, TableHandle, TableInfo,
    UpdateRowsRequestV1, UpdateRowsResponseV1, VectorSearchRequestV1, VersionInfoV1,
    WriteDataMode, WriteRowsRequestV1, WriteRowsResponseV1, AuthDescriptor,
};
use crate::state::AppState;

fn batches_to_json_rows(batches: &[RecordBatch]) -> Result<Vec<serde_json::Value>, String> {
    if batches.is_empty() {
        return Ok(Vec::new());
    }

    let mut writer = ArrayWriter::new(Vec::new());
    let batch_refs: Vec<&RecordBatch> = batches.iter().collect();

    writer
        .write_batches(&batch_refs)
        .map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())?;

    let json = writer.into_inner();
    let rows: Vec<serde_json::Value> =
        serde_json::from_slice(&json).map_err(|error| error.to_string())?;

    Ok(rows)
}

fn batches_to_arrow_ipc_base64(batches: &[RecordBatch], schema: &Schema) -> Result<String, String> {
    let mut buffer = Vec::new();
    let mut writer = StreamWriter::try_new(&mut buffer, schema)
        .map_err(|error| error.to_string())?;

    for batch in batches {
        writer.write(batch).map_err(|error| error.to_string())?;
    }

    writer.finish().map_err(|error| error.to_string())?;
    Ok(general_purpose::STANDARD.encode(buffer))
}

fn truncate_batches(batches: &[RecordBatch], limit: usize) -> Vec<RecordBatch> {
    if limit == 0 {
        return Vec::new();
    }

    let mut remaining = limit;
    let mut trimmed = Vec::new();

    for batch in batches {
        if remaining == 0 {
            break;
        }
        let rows = batch.num_rows();
        if rows <= remaining {
            trimmed.push(batch.clone());
            remaining = remaining.saturating_sub(rows);
        } else {
            trimmed.push(batch.slice(0, remaining));
            remaining = 0;
        }
    }

    trimmed
}

#[derive(Debug, Clone, Default)]
struct QueryOptions {
    projection: Option<Vec<String>>,
    filter: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

fn apply_query_options<Q: QueryBase>(mut query: Q, options: &QueryOptions) -> Q {
    if let Some(filter) = options.filter.as_deref() {
        query = query.only_if(filter);
    }

    if let Some(limit) = options.limit {
        query = query.limit(limit);
    }

    if let Some(offset) = options.offset {
        query = query.offset(offset);
    }

    if let Some(projection) = &options.projection {
        if !projection.is_empty() {
            query = query.select(Select::columns(projection));
        }
    }

    query
}

async fn execute_query_json(
    query: impl ExecutableQuery,
    fallback_schema: SchemaDefinition,
) -> Result<(Vec<serde_json::Value>, SchemaDefinition), String> {
    let batches = execute_query_batches(query).await?;
    let batch_count = batches.len();

    let schema = if let Some(first) = batches.first() {
        SchemaDefinition::from_arrow_schema(first.schema().as_ref())
    } else {
        fallback_schema
    };

    let rows = batches_to_json_rows(&batches)?;
    trace!(
        "execute_query_json completed batches={} rows={}",
        batch_count,
        rows.len()
    );
    Ok((rows, schema))
}

async fn execute_query_batches(query: impl ExecutableQuery) -> Result<Vec<RecordBatch>, String> {
    let stream = query.execute().await.map_err(|error| error.to_string())?;
    stream
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| error.to_string())
}

fn json_rows_to_batches(schema: SchemaRef, rows: &[serde_json::Value]) -> Result<Vec<RecordBatch>, String> {
    if rows.is_empty() {
        return Err("rows cannot be empty".to_string());
    }
    let mut buffer = String::new();
    for row in rows {
        let line = serde_json::to_string(row).map_err(|error| error.to_string())?;
        buffer.push_str(&line);
        buffer.push('\n');
    }
    let cursor = Cursor::new(buffer.into_bytes());
    let mut reader = ReaderBuilder::new(schema).build(cursor).map_err(|error| error.to_string())?;
    let mut batches = Vec::new();
    while let Some(batch) = reader.next() {
        let batch = batch.map_err(|error| error.to_string())?;
        batches.push(batch);
    }
    if batches.is_empty() {
        return Err("no rows parsed from input".to_string());
    }
    Ok(batches)
}

fn parse_delimiter(delimiter: Option<String>, fallback: u8) -> Result<u8, String> {
    let Some(value) = delimiter else {
        return Ok(fallback);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(fallback);
    }
    let bytes = trimmed.as_bytes();
    if bytes.len() != 1 {
        return Err("delimiter must be a single character".to_string());
    }
    Ok(bytes[0])
}

fn sanitize_filter(filter: Option<String>) -> Option<String> {
    filter.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn sanitize_projection(projection: Option<Vec<String>>) -> Option<Vec<String>> {
    let values = projection?;
    let cleaned = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn to_arrow_data_type(
    data_type: &FieldDataType,
    vector_length: Option<i32>,
) -> Result<DataType, String> {
    match data_type {
        FieldDataType::Int8 => Ok(DataType::Int8),
        FieldDataType::Int16 => Ok(DataType::Int16),
        FieldDataType::Int32 => Ok(DataType::Int32),
        FieldDataType::Int64 => Ok(DataType::Int64),
        FieldDataType::UInt8 => Ok(DataType::UInt8),
        FieldDataType::UInt16 => Ok(DataType::UInt16),
        FieldDataType::UInt32 => Ok(DataType::UInt32),
        FieldDataType::UInt64 => Ok(DataType::UInt64),
        FieldDataType::Float32 => Ok(DataType::Float32),
        FieldDataType::Float64 => Ok(DataType::Float64),
        FieldDataType::Boolean => Ok(DataType::Boolean),
        FieldDataType::Utf8 => Ok(DataType::Utf8),
        FieldDataType::LargeUtf8 => Ok(DataType::LargeUtf8),
        FieldDataType::Binary => Ok(DataType::Binary),
        FieldDataType::LargeBinary => Ok(DataType::LargeBinary),
        FieldDataType::FixedSizeListFloat32 => {
            let length = vector_length.ok_or_else(|| {
                "vector_length is required for fixed_size_list_float32".to_string()
            })?;
            if length <= 0 {
                return Err("vector_length must be greater than 0".to_string());
            }
            let item_field = Arc::new(Field::new("item", DataType::Float32, true));
            Ok(DataType::FixedSizeList(item_field, length))
        }
    }
}

fn to_arrow_field(input: &SchemaFieldInput) -> Result<Field, String> {
    let data_type = to_arrow_data_type(&input.data_type, input.vector_length)?;
    let mut field = Field::new(&input.name, data_type, input.nullable);
    if let Some(metadata) = &input.metadata {
        field = field.with_metadata(metadata.clone());
    }
    Ok(field)
}

fn to_arrow_schema(input: &SchemaDefinitionInput) -> Result<SchemaRef, String> {
    if input.fields.is_empty() {
        return Err("schema must contain at least one field".to_string());
    }
    let fields = input
        .fields
        .iter()
        .map(to_arrow_field)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(Schema::new(fields)))
}

fn sanitize_index_columns(columns: &[String]) -> Result<Vec<String>, String> {
    if columns.is_empty() {
        return Err("columns cannot be empty".to_string());
    }

    let mut cleaned = Vec::new();
    for column in columns {
        let trimmed = column.trim();
        if trimmed.is_empty() {
            return Err("index column cannot be empty".to_string());
        }
        if !cleaned.iter().any(|value: &String| value == trimmed) {
            cleaned.push(trimmed.to_string());
        }
    }

    if cleaned.is_empty() {
        return Err("columns cannot be empty".to_string());
    }

    Ok(cleaned)
}

fn to_index_type_v1(index_type: &IndexType) -> IndexTypeV1 {
    match index_type {
        IndexType::BTree => IndexTypeV1::BTree,
        IndexType::Bitmap => IndexTypeV1::Bitmap,
        IndexType::LabelList => IndexTypeV1::LabelList,
        IndexType::FTS => IndexTypeV1::Fts,
        IndexType::IvfFlat => IndexTypeV1::IvfFlat,
        IndexType::IvfSq => IndexTypeV1::IvfSq,
        IndexType::IvfPq => IndexTypeV1::IvfPq,
        IndexType::IvfRq => IndexTypeV1::IvfRq,
        IndexType::IvfHnswPq => IndexTypeV1::IvfHnswPq,
        IndexType::IvfHnswSq => IndexTypeV1::IvfHnswSq,
    }
}

fn to_lancedb_index(index_type: &IndexTypeV1) -> Index {
    match index_type {
        IndexTypeV1::Auto => Index::Auto,
        IndexTypeV1::BTree => Index::BTree(BTreeIndexBuilder::default()),
        IndexTypeV1::Bitmap => Index::Bitmap(BitmapIndexBuilder::default()),
        IndexTypeV1::LabelList => Index::LabelList(LabelListIndexBuilder::default()),
        IndexTypeV1::Fts => Index::FTS(FtsIndexBuilder::default()),
        IndexTypeV1::IvfFlat => Index::IvfFlat(IvfFlatIndexBuilder::default()),
        IndexTypeV1::IvfSq => Index::IvfSq(IvfSqIndexBuilder::default()),
        IndexTypeV1::IvfPq => Index::IvfPq(IvfPqIndexBuilder::default()),
        IndexTypeV1::IvfRq => Index::IvfRq(IvfRqIndexBuilder::default()),
        IndexTypeV1::IvfHnswPq => Index::IvfHnswPq(IvfHnswPqIndexBuilder::default()),
        IndexTypeV1::IvfHnswSq => Index::IvfHnswSq(IvfHnswSqIndexBuilder::default()),
    }
}

async fn read_table_schema(table: &Table) -> Result<SchemaDefinition, String> {
    let schema = table.schema().await.map_err(|error| error.to_string())?;
    Ok(SchemaDefinition::from_arrow_schema(schema.as_ref()))
}

fn to_version_info(version: lancedb::table::Version) -> VersionInfoV1 {
    VersionInfoV1 {
        version: version.version,
        timestamp: version.timestamp.to_rfc3339(),
        metadata: version.metadata.into_iter().collect(),
    }
}

pub async fn connect_v1(
    state: &AppState,
    request: ConnectRequestV1,
) -> ResultEnvelope<ConnectResponseV1> {
    let started_at = Instant::now();
    let profile = request.profile;
    let backend_kind = infer_backend_kind(&profile.uri);
    let mut storage_options = profile.storage_options.clone();

    info!(
        "connect_v1 start name=\"{}\" uri=\"{}\" backend={:?}",
        profile.name, profile.uri, backend_kind
    );
    match &profile.auth {
        AuthDescriptor::None => {}
        AuthDescriptor::Inline { provider, params } => {
            if !params.is_empty() {
                let keys: Vec<String> = params.keys().cloned().collect();
                trace!("connect_v1 auth_provider=\"{}\" auth_keys={:?}", provider, keys);
            }
            for (key, value) in params {
                storage_options.insert(key.clone(), value.clone());
            }
        }
        AuthDescriptor::SecretRef { provider, reference } => {
            warn!(
                "connect_v1 secret_ref not supported provider=\"{}\" reference=\"{}\"",
                provider, reference
            );
            return ResultEnvelope::err(
                ErrorCode::NotImplemented,
                "secret_ref auth is not supported; resolve it before connecting",
            );
        }
    }

    if !storage_options.is_empty() {
        let keys: Vec<String> = storage_options.keys().cloned().collect();
        trace!("connect_v1 storage_options_keys={:?}", keys);
    }
    if let Some(interval) = profile.options.read_consistency_interval_seconds {
        debug!("connect_v1 read_consistency_interval_seconds={}", interval);
    }

    let mut builder = lancedb::connect(&profile.uri);
    if !storage_options.is_empty() {
        builder = builder.storage_options(
            storage_options
                .iter()
                .map(|(key, value)| (key.clone(), value.clone())),
        );
    }
    if let Some(interval) = profile.options.read_consistency_interval_seconds {
        builder = builder.read_consistency_interval(Duration::from_secs(interval));
    }

    let connection = match builder.execute().await {
        Ok(connection) => connection,
        Err(error) => {
            error!(
                "connect_v1 failed to connect uri=\"{}\" error={}",
                profile.uri, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let connection_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_connection(connection),
        Err(_) => {
            error!("connect_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    info!(
        "connect_v1 ok id={} backend={:?} elapsed_ms={}",
        connection_id,
        backend_kind,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(ConnectResponseV1 {
        connection_id,
        backend_kind,
        name: profile.name,
        uri: profile.uri,
    })
}

pub async fn disconnect_v1(
    state: &AppState,
    request: DisconnectRequestV1,
) -> ResultEnvelope<DisconnectResponseV1> {
    let started_at = Instant::now();
    info!(
        "disconnect_v1 start connection_id={}",
        request.connection_id
    );

    let removed_tables = match state.connections.lock() {
        Ok(mut manager) => match manager.remove_connection(&request.connection_id) {
            Some(count) => count,
            None => {
                warn!(
                    "disconnect_v1 connection not found connection_id={}",
                    request.connection_id
                );
                return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
            }
        },
        Err(_) => {
            error!("disconnect_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    info!(
        "disconnect_v1 ok connection_id={} released_tables={} elapsed_ms={}",
        request.connection_id,
        removed_tables,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(DisconnectResponseV1 {
        connection_id: request.connection_id,
        released_tables: removed_tables,
    })
}

pub async fn list_tables_v1(
    state: &AppState,
    request: ListTablesRequestV1,
) -> ResultEnvelope<ListTablesResponseV1> {
    let started_at = Instant::now();
    info!("list_tables_v1 start connection_id={}", request.connection_id);
    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("list_tables_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(connection) = connection else {
        warn!(
            "list_tables_v1 connection not found connection_id={}",
            request.connection_id
        );
        return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
    };

    let names: Vec<String> = match connection.table_names().execute().await {
        Ok(names) => names,
        Err(error) => {
            error!(
                "list_tables_v1 failed connection_id={} error={} ",
                request.connection_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let tables: Vec<TableInfo> = names.into_iter().map(|name| TableInfo { name }).collect();

    info!(
        "list_tables_v1 ok connection_id={} tables={} elapsed_ms={}",
        request.connection_id,
        tables.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(ListTablesResponseV1 { tables })
}

pub async fn drop_table_v1(
    state: &AppState,
    request: DropTableRequestV1,
) -> ResultEnvelope<DropTableResponseV1> {
    let started_at = Instant::now();
    info!(
        "drop_table_v1 start connection_id={} table=\"{}\"",
        request.connection_id, request.table_name
    );

    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("drop_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(connection) = connection else {
        warn!(
            "drop_table_v1 connection not found connection_id={}",
            request.connection_id
        );
        return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
    };

    let namespace = request.namespace.unwrap_or_default();
    if let Err(error) = connection.drop_table(&request.table_name, &namespace).await {
        error!(
            "drop_table_v1 failed connection_id={} table=\"{}\" error={}",
            request.connection_id, request.table_name, error
        );
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    info!(
        "drop_table_v1 ok connection_id={} table=\"{}\" elapsed_ms={}",
        request.connection_id,
        request.table_name,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(DropTableResponseV1 {
        table_name: request.table_name,
    })
}

pub async fn rename_table_v1(
    state: &AppState,
    request: RenameTableRequestV1,
) -> ResultEnvelope<RenameTableResponseV1> {
    let started_at = Instant::now();
    info!(
        "rename_table_v1 start connection_id={} table=\"{}\"",
        request.connection_id, request.table_name
    );

    let table_name = request.table_name.trim();
    if table_name.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "table name cannot be empty");
    }

    let new_table_name = request.new_table_name.trim();
    if new_table_name.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "new table name cannot be empty");
    }

    if table_name == new_table_name {
        return ResultEnvelope::err(
            ErrorCode::InvalidArgument,
            "new table name must differ from the current name",
        );
    }

    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("rename_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(connection) = connection else {
        warn!(
            "rename_table_v1 connection not found connection_id={}",
            request.connection_id
        );
        return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
    };

    let namespace = request.namespace.unwrap_or_default();
    let new_namespace = request.new_namespace.unwrap_or_default();

    if let Err(error) = connection
        .rename_table(table_name, new_table_name, &namespace, &new_namespace)
        .await
    {
        let message = error.to_string();
        let lower = message.to_lowercase();
        let code = if lower.contains("not supported") {
            ErrorCode::NotImplemented
        } else {
            ErrorCode::Internal
        };
        error!(
            "rename_table_v1 failed connection_id={} table=\"{}\" error={}",
            request.connection_id, table_name, message
        );
        return ResultEnvelope::err(code, message);
    }

    info!(
        "rename_table_v1 ok connection_id={} table=\"{}\" new_table=\"{}\" elapsed_ms={}",
        request.connection_id,
        table_name,
        new_table_name,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(RenameTableResponseV1 {
        table_name: table_name.to_string(),
        new_table_name: new_table_name.to_string(),
    })
}

pub async fn list_indexes_v1(
    state: &AppState,
    request: ListIndexesRequestV1,
) -> ResultEnvelope<ListIndexesResponseV1> {
    let started_at = Instant::now();
    info!("list_indexes_v1 start table_id={}", request.table_id);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("list_indexes_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("list_indexes_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let index_configs = match table.list_indices().await {
        Ok(configs) => configs,
        Err(error) => {
            error!(
                "list_indexes_v1 failed table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let indexes = index_configs
        .into_iter()
        .map(|config| IndexDefinitionV1 {
            name: config.name,
            index_type: to_index_type_v1(&config.index_type),
            columns: config.columns,
        })
        .collect::<Vec<_>>();

    info!(
        "list_indexes_v1 ok table_id={} indexes={} elapsed_ms={}",
        request.table_id,
        indexes.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(ListIndexesResponseV1 { indexes })
}

pub async fn create_index_v1(
    state: &AppState,
    request: CreateIndexRequestV1,
) -> ResultEnvelope<CreateIndexResponseV1> {
    let started_at = Instant::now();
    info!(
        "create_index_v1 start table_id={} columns={} index_type={:?}",
        request.table_id,
        request.columns.len(),
        request.index_type
    );

    let columns = match sanitize_index_columns(&request.columns) {
        Ok(columns) => columns,
        Err(error) => {
            warn!("create_index_v1 invalid columns error={}", error);
            return ResultEnvelope::err(ErrorCode::InvalidArgument, error);
        }
    };

    let name = request
        .name
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    if request.name.is_some() && name.is_none() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "index name cannot be empty");
    }
    let resolved_name = name.map(str::to_string);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("create_index_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("create_index_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let index = to_lancedb_index(&request.index_type);
    let mut builder = table.create_index(&columns, index).replace(request.replace);
    if let Some(name) = resolved_name.as_ref() {
        builder = builder.name(name.clone());
    }

    if let Err(error) = builder.execute().await {
        error!(
            "create_index_v1 failed table_id={} error={}",
            request.table_id, error
        );
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    info!(
        "create_index_v1 ok table_id={} elapsed_ms={}",
        request.table_id,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(CreateIndexResponseV1 {
        table_id: request.table_id,
        index_type: request.index_type,
        columns,
        name: resolved_name,
    })
}

pub async fn drop_index_v1(
    state: &AppState,
    request: DropIndexRequestV1,
) -> ResultEnvelope<DropIndexResponseV1> {
    let started_at = Instant::now();
    info!(
        "drop_index_v1 start table_id={} index_name=\"{}\"",
        request.table_id, request.index_name
    );

    let index_name = request.index_name.trim();
    if index_name.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "index name cannot be empty");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("drop_index_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("drop_index_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    if let Err(error) = table.drop_index(index_name).await {
        error!(
            "drop_index_v1 failed table_id={} error={}",
            request.table_id, error
        );
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    info!(
        "drop_index_v1 ok table_id={} elapsed_ms={}",
        request.table_id,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(DropIndexResponseV1 {
        table_id: request.table_id,
        index_name: index_name.to_string(),
    })
}

pub async fn create_table_v1(
    state: &AppState,
    request: CreateTableRequestV1,
) -> ResultEnvelope<CreateTableResponseV1> {
    let started_at = Instant::now();
    info!(
        "create_table_v1 start connection_id={} table=\"{}\"",
        request.connection_id, request.table_name
    );

    if request.table_name.trim().is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "table name cannot be empty");
    }

    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("create_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(connection) = connection else {
        warn!(
            "create_table_v1 connection not found connection_id={}",
            request.connection_id
        );
        return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
    };

    let schema = match to_arrow_schema(&request.schema) {
        Ok(schema) => schema,
        Err(error) => {
            warn!("create_table_v1 invalid schema error={}", error);
            return ResultEnvelope::err(ErrorCode::InvalidArgument, error);
        }
    };

    let table = match connection
        .create_empty_table(&request.table_name, schema)
        .execute()
        .await
    {
        Ok(table) => table,
        Err(error) => {
            error!(
                "create_table_v1 failed connection_id={} table=\"{}\" error={}",
                request.connection_id, request.table_name, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let table_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_table(
            request.table_name.clone(),
            table,
            request.connection_id.clone(),
        ),
        Err(_) => {
            error!("create_table_v1 failed to lock table manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock table manager");
        }
    };

    info!(
        "create_table_v1 ok connection_id={} table_id={} table=\"{}\" elapsed_ms={}",
        request.connection_id,
        table_id,
        request.table_name,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(CreateTableResponseV1 {
        table_id,
        name: request.table_name,
    })
}

pub async fn add_columns_v1(
    state: &AppState,
    request: AddColumnsRequestV1,
) -> ResultEnvelope<AddColumnsResponseV1> {
    let started_at = Instant::now();
    info!("add_columns_v1 start table_id={}", request.table_id);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("add_columns_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("add_columns_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let schema = match to_arrow_schema(&request.columns) {
        Ok(schema) => schema,
        Err(error) => {
            warn!("add_columns_v1 invalid schema error={}", error);
            return ResultEnvelope::err(ErrorCode::InvalidArgument, error);
        }
    };

    let transforms = NewColumnTransform::AllNulls(schema);
    if let Err(error) = table.add_columns(transforms, None).await {
        error!("add_columns_v1 failed table_id={} error={}", request.table_id, error);
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    let updated_schema = match read_table_schema(&table).await {
        Ok(schema) => schema,
        Err(error) => {
            error!("add_columns_v1 schema reload failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };

    let added = request
        .columns
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();

    info!(
        "add_columns_v1 ok table_id={} added={} elapsed_ms={}",
        request.table_id,
        added.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(AddColumnsResponseV1 {
        table_id: request.table_id,
        added,
        schema: updated_schema,
    })
}

fn build_column_alteration(input: &ColumnAlterationInput) -> Result<ColumnAlteration, String> {
    if input.path.trim().is_empty() {
        return Err("column path cannot be empty".to_string());
    }
    let has_change = input
        .rename
        .as_ref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || input.nullable.is_some()
        || input.data_type.is_some();
    if !has_change {
        return Err("column alteration must specify rename, nullable, or data_type".to_string());
    }
    let mut alteration = ColumnAlteration::new(input.path.trim().to_string());
    if let Some(rename) = input.rename.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty()) {
        alteration = alteration.rename(rename.to_string());
    }
    if let Some(nullable) = input.nullable {
        alteration = alteration.set_nullable(nullable);
    }
    if let Some(data_type) = input.data_type.as_ref() {
        let arrow_type = to_arrow_data_type(data_type, input.vector_length)?;
        alteration = alteration.cast_to(arrow_type);
    }
    Ok(alteration)
}

pub async fn alter_columns_v1(
    state: &AppState,
    request: AlterColumnsRequestV1,
) -> ResultEnvelope<AlterColumnsResponseV1> {
    let started_at = Instant::now();
    info!("alter_columns_v1 start table_id={}", request.table_id);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("alter_columns_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("alter_columns_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    if request.columns.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "no column alterations provided");
    }

    let mut updated_paths = Vec::new();
    let alterations = match request
        .columns
        .iter()
        .map(|input| {
            let alteration = build_column_alteration(input)?;
            updated_paths.push(alteration.path.clone());
            Ok(alteration)
        })
        .collect::<Result<Vec<_>, String>>()
    {
        Ok(result) => result,
        Err(error) => {
            warn!("alter_columns_v1 invalid alteration error={}", error);
            return ResultEnvelope::err(ErrorCode::InvalidArgument, error);
        }
    };

    if let Err(error) = table.alter_columns(&alterations).await {
        error!("alter_columns_v1 failed table_id={} error={}", request.table_id, error);
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    let updated_schema = match read_table_schema(&table).await {
        Ok(schema) => schema,
        Err(error) => {
            error!("alter_columns_v1 schema reload failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };

    info!(
        "alter_columns_v1 ok table_id={} updated={} elapsed_ms={}",
        request.table_id,
        updated_paths.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(AlterColumnsResponseV1 {
        table_id: request.table_id,
        updated: updated_paths,
        schema: updated_schema,
    })
}

pub async fn drop_columns_v1(
    state: &AppState,
    request: DropColumnsRequestV1,
) -> ResultEnvelope<DropColumnsResponseV1> {
    let started_at = Instant::now();
    info!("drop_columns_v1 start table_id={}", request.table_id);

    if request.columns.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "no columns specified");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("drop_columns_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("drop_columns_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let column_refs = request
        .columns
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if let Err(error) = table.drop_columns(&column_refs).await {
        error!("drop_columns_v1 failed table_id={} error={}", request.table_id, error);
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    let updated_schema = match read_table_schema(&table).await {
        Ok(schema) => schema,
        Err(error) => {
            error!("drop_columns_v1 schema reload failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };

    info!(
        "drop_columns_v1 ok table_id={} dropped={} elapsed_ms={}",
        request.table_id,
        request.columns.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(DropColumnsResponseV1 {
        table_id: request.table_id,
        dropped: request.columns,
        schema: updated_schema,
    })
}

pub async fn write_rows_v1(
    state: &AppState,
    request: WriteRowsRequestV1,
) -> ResultEnvelope<WriteRowsResponseV1> {
    let started_at = Instant::now();
    info!(
        "write_rows_v1 start table_id={} rows={} mode={:?}",
        request.table_id,
        request.rows.len(),
        request.mode
    );

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("write_rows_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("write_rows_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let schema = match table.schema().await {
        Ok(schema) => schema,
        Err(error) => {
            error!("write_rows_v1 failed to read schema table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let batches = match json_rows_to_batches(schema.clone(), &request.rows) {
        Ok(batches) => batches,
        Err(error) => {
            warn!("write_rows_v1 invalid rows table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::InvalidArgument, error);
        }
    };

    let batch_iter = RecordBatchIterator::new(batches.into_iter().map(Ok), schema.clone());
    let mut builder = table.add(batch_iter);
    if matches!(request.mode, WriteDataMode::Overwrite) {
        builder = builder.mode(AddDataMode::Overwrite);
    }

    let result = match builder.execute().await {
        Ok(result) => result,
        Err(error) => {
            error!("write_rows_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "write_rows_v1 ok table_id={} rows={} version={} elapsed_ms={}",
        request.table_id,
        request.rows.len(),
        result.version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(WriteRowsResponseV1 {
        table_id: request.table_id,
        rows: request.rows.len(),
        version: result.version,
    })
}

pub async fn update_rows_v1(
    state: &AppState,
    request: UpdateRowsRequestV1,
) -> ResultEnvelope<UpdateRowsResponseV1> {
    let started_at = Instant::now();
    info!(
        "update_rows_v1 start table_id={} updates={}",
        request.table_id,
        request.updates.len()
    );

    if request.updates.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "no updates specified");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("update_rows_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("update_rows_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let mut builder = table.update();
    if let Some(filter) = request.filter.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty()) {
        builder = builder.only_if(filter.to_string());
    }

    for update in &request.updates {
        let column = update.column.trim();
        let expr = update.expr.trim();
        if column.is_empty() || expr.is_empty() {
            return ResultEnvelope::err(
                ErrorCode::InvalidArgument,
                "update column and expression cannot be empty",
            );
        }
        builder = builder.column(column.to_string(), expr.to_string());
    }

    let result = match builder.execute().await {
        Ok(result) => result,
        Err(error) => {
            error!("update_rows_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "update_rows_v1 ok table_id={} rows_updated={} version={} elapsed_ms={}",
        request.table_id,
        result.rows_updated,
        result.version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(UpdateRowsResponseV1 {
        table_id: request.table_id,
        rows_updated: result.rows_updated,
        version: result.version,
    })
}

pub async fn delete_rows_v1(
    state: &AppState,
    request: DeleteRowsRequestV1,
) -> ResultEnvelope<DeleteRowsResponseV1> {
    let started_at = Instant::now();
    info!("delete_rows_v1 start table_id={}", request.table_id);

    let filter = request.filter.trim();
    if filter.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "filter cannot be empty");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("delete_rows_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("delete_rows_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let result = match table.delete(filter).await {
        Ok(result) => result,
        Err(error) => {
            error!("delete_rows_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "delete_rows_v1 ok table_id={} version={} elapsed_ms={}",
        request.table_id,
        result.version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(DeleteRowsResponseV1 {
        table_id: request.table_id,
        version: result.version,
    })
}

pub async fn import_data_v1(
    state: &AppState,
    request: ImportDataRequestV1,
) -> ResultEnvelope<ImportDataResponseV1> {
    let started_at = Instant::now();
    let path = request.path.trim();
    info!(
        "import_data_v1 start table_id={} format={:?} path=\"{}\"",
        request.table_id, request.format, path
    );
    if path.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "path cannot be empty");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("import_data_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("import_data_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let schema = match table.schema().await {
        Ok(schema) => schema,
        Err(error) => {
            error!("import_data_v1 failed to read schema table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let (batches, total_rows) = match request.format {
        DataFileFormatV1::Csv => {
            let has_header = request.has_header.unwrap_or(true);
            let delimiter = match parse_delimiter(request.delimiter.clone(), b',') {
                Ok(delimiter) => delimiter,
                Err(error) => return ResultEnvelope::err(ErrorCode::InvalidArgument, error),
            };
            let file = match File::open(path) {
                Ok(file) => file,
                Err(error) => {
                    return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
                }
            };
            let mut reader = match CsvReaderBuilder::new(schema.clone())
                .with_header(has_header)
                .with_delimiter(delimiter)
                .build(file)
            {
                Ok(reader) => reader,
                Err(error) => {
                    return ResultEnvelope::err(ErrorCode::InvalidArgument, error.to_string());
                }
            };
            let mut batches = Vec::new();
            while let Some(batch) = reader.next() {
                let batch = match batch {
                    Ok(batch) => batch,
                    Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
                };
                batches.push(batch);
            }
            let total = batches.iter().map(|batch| batch.num_rows()).sum::<usize>();
            (batches, total)
        }
        DataFileFormatV1::Parquet => {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(error) => {
                    return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
                }
            };
            let mut reader = match ParquetRecordBatchReaderBuilder::try_new(file)
                .and_then(|builder| builder.build())
            {
                Ok(reader) => reader,
                Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
            };
            let mut batches = Vec::new();
            while let Some(batch) = reader.next() {
                let batch = match batch {
                    Ok(batch) => batch,
                    Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
                };
                batches.push(batch);
            }
            let total = batches.iter().map(|batch| batch.num_rows()).sum::<usize>();
            (batches, total)
        }
        DataFileFormatV1::Jsonl => {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(error) => {
                    return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
                }
            };
            let reader = BufReader::new(file);
            let mut rows = Vec::new();
            for line in reader.lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
                };
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let value = match serde_json::from_str::<serde_json::Value>(trimmed) {
                    Ok(value) => value,
                    Err(error) => {
                        return ResultEnvelope::err(ErrorCode::InvalidArgument, error.to_string())
                    }
                };
                rows.push(value);
            }
            if rows.is_empty() {
                return ResultEnvelope::err(ErrorCode::InvalidArgument, "no rows found in file");
            }
            let batches = match json_rows_to_batches(schema.clone(), &rows) {
                Ok(batches) => batches,
                Err(error) => return ResultEnvelope::err(ErrorCode::InvalidArgument, error),
            };
            let total = batches.iter().map(|batch| batch.num_rows()).sum::<usize>();
            (batches, total)
        }
    };

    if batches.is_empty() || total_rows == 0 {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "no rows to import");
    }

    let schema_for_batches = batches
        .first()
        .map(|batch| batch.schema())
        .unwrap_or_else(|| schema.clone());
    let batch_iter = RecordBatchIterator::new(batches.into_iter().map(Ok), schema_for_batches);
    let mut builder = table.add(batch_iter);
    if matches!(request.mode, WriteDataMode::Overwrite) {
        builder = builder.mode(AddDataMode::Overwrite);
    }

    let result = match builder.execute().await {
        Ok(result) => result,
        Err(error) => {
            error!("import_data_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "import_data_v1 ok table_id={} rows={} version={} elapsed_ms={}",
        request.table_id,
        total_rows,
        result.version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(ImportDataResponseV1 {
        table_id: request.table_id,
        rows: total_rows,
    })
}

pub async fn export_data_v1(
    state: &AppState,
    request: ExportDataRequestV1,
) -> ResultEnvelope<ExportDataResponseV1> {
    let started_at = Instant::now();
    let path = request.path.trim();
    info!(
        "export_data_v1 start table_id={} format={:?} path=\"{}\"",
        request.table_id, request.format, path
    );
    if path.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "path cannot be empty");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("export_data_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("export_data_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let fallback_schema = match table.schema().await {
        Ok(schema) => schema,
        Err(error) => {
            error!("export_data_v1 failed to read schema table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let options = QueryOptions {
        projection: sanitize_projection(request.projection.clone()),
        filter: sanitize_filter(request.filter.clone()),
        limit: request.limit,
        offset: request.offset,
    };

    let query = apply_query_options(table.query(), &options);
    let batches = match execute_query_batches(query).await {
        Ok(batches) => batches,
        Err(error) => {
            error!("export_data_v1 query failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };
    let total_rows = batches.iter().map(|batch| batch.num_rows()).sum::<usize>();

    match request.format {
        DataFileFormatV1::Csv => {
            let delimiter = match parse_delimiter(request.delimiter.clone(), b',') {
                Ok(delimiter) => delimiter,
                Err(error) => return ResultEnvelope::err(ErrorCode::InvalidArgument, error),
            };
            let with_header = request.with_header.unwrap_or(true);
            let file = match File::create(path) {
                Ok(file) => file,
                Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
            };
            let mut writer = CsvWriterBuilder::new()
                .with_header(with_header)
                .with_delimiter(delimiter)
                .build(BufWriter::new(file));
            if batches.is_empty() {
                let empty_batch = RecordBatch::new_empty(fallback_schema.clone());
                if let Err(error) = writer.write(&empty_batch) {
                    return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
                }
            } else {
                for batch in &batches {
                    if let Err(error) = writer.write(batch) {
                        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
                    }
                }
            }
        }
        DataFileFormatV1::Parquet => {
            let file = match File::create(path) {
                Ok(file) => file,
                Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
            };
            let schema = batches
                .first()
                .map(|batch| batch.schema())
                .unwrap_or_else(|| fallback_schema.clone());
            let mut writer = match ArrowWriter::try_new(file, schema, None) {
                Ok(writer) => writer,
                Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
            };
            for batch in &batches {
                if let Err(error) = writer.write(batch) {
                    return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
                }
            }
            if let Err(error) = writer.close() {
                return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
            }
        }
        DataFileFormatV1::Jsonl => {
            let file = match File::create(path) {
                Ok(file) => file,
                Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
            };
            let mut writer = BufWriter::new(file);
            let rows = match batches_to_json_rows(&batches) {
                Ok(rows) => rows,
                Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error),
            };
            for row in rows {
                let line = match serde_json::to_string(&row) {
                    Ok(line) => line,
                    Err(error) => return ResultEnvelope::err(ErrorCode::Internal, error.to_string()),
                };
                if writer.write_all(line.as_bytes()).is_err()
                    || writer.write_all(b"\n").is_err()
                {
                    return ResultEnvelope::err(ErrorCode::Internal, "failed to write jsonl".to_string());
                }
            }
            if writer.flush().is_err() {
                return ResultEnvelope::err(ErrorCode::Internal, "failed to flush jsonl".to_string());
            }
        }
    }

    info!(
        "export_data_v1 ok table_id={} rows={} elapsed_ms={}",
        request.table_id,
        total_rows,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(ExportDataResponseV1 {
        path: request.path,
        rows: total_rows,
    })
}

pub async fn optimize_table_v1(
    state: &AppState,
    request: OptimizeTableRequestV1,
) -> ResultEnvelope<OptimizeTableResponseV1> {
    let started_at = Instant::now();
    info!(
        "optimize_table_v1 start table_id={} action={:?}",
        request.table_id, request.action
    );

    let OptimizeTableRequestV1 {
        table_id,
        action,
        target_rows_per_fragment,
        older_than_days,
        delete_unverified,
        error_if_tagged_old_versions,
    } = request;

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&table_id),
        Err(_) => {
            error!("optimize_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("optimize_table_v1 table not found table_id={}", table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let (opt_action, summary) = match action {
        OptimizeActionV1::Compact => {
            if let Some(target_rows) = target_rows_per_fragment {
                if target_rows == 0 {
                    return ResultEnvelope::err(
                        ErrorCode::InvalidArgument,
                        "target_rows_per_fragment must be greater than 0",
                    );
                }
            }
            let mut options = CompactionOptions::default();
            if let Some(target_rows) = target_rows_per_fragment {
                let target_rows = match usize::try_from(target_rows) {
                    Ok(value) => value,
                    Err(_) => {
                        return ResultEnvelope::err(
                            ErrorCode::InvalidArgument,
                            "target_rows_per_fragment is too large",
                        );
                    }
                };
                options.target_rows_per_fragment = target_rows;
            }
            let summary = target_rows_per_fragment
                .map(|value| format!("Compact 已提交，目标片段行数={value}"))
                .unwrap_or_else(|| "Compact 已提交".to_string());
            (
                OptimizeAction::Compact {
                    options,
                    remap_options: None,
                },
                summary,
            )
        }
        OptimizeActionV1::Vacuum => {
            let older_than = match older_than_days {
                Some(days) => {
                    let days_i64 = match i64::try_from(days) {
                        Ok(value) => value,
                        Err(_) => {
                            return ResultEnvelope::err(
                                ErrorCode::InvalidArgument,
                                "older_than_days is too large",
                            );
                        }
                    };
                    Some(LanceDuration::days(days_i64))
                }
                None => None,
            };
            let summary = older_than_days
                .map(|value| format!("Vacuum 已提交，清理超过 {value} 天的历史版本"))
                .unwrap_or_else(|| "Vacuum 已提交".to_string());
            (
                OptimizeAction::Prune {
                    older_than,
                    delete_unverified,
                    error_if_tagged_old_versions,
                },
                summary,
            )
        }
    };

    if let Err(error) = table.optimize(opt_action).await {
        let message = error.to_string();
        let lower = message.to_lowercase();
        let code = if lower.contains("not supported") {
            ErrorCode::NotImplemented
        } else {
            ErrorCode::Internal
        };
        error!(
            "optimize_table_v1 failed table_id={} error={}",
            table_id, message
        );
        return ResultEnvelope::err(code, message);
    }

    info!(
        "optimize_table_v1 ok table_id={} action={:?} elapsed_ms={}",
        table_id,
        action,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(OptimizeTableResponseV1 {
        table_id,
        action,
        summary,
    })
}

pub async fn open_table_v1(
    state: &AppState,
    request: OpenTableRequestV1,
) -> ResultEnvelope<TableHandle> {
    let started_at = Instant::now();
    info!(
        "open_table_v1 start connection_id={} table=\"{}\"",
        request.connection_id, request.table_name
    );
    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("open_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(connection) = connection else {
        warn!(
            "open_table_v1 connection not found connection_id={}",
            request.connection_id
        );
        return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
    };

    let table = match connection.open_table(&request.table_name).execute().await {
        Ok(table) => table,
        Err(error) => {
            error!(
                "open_table_v1 failed connection_id={} table=\"{}\" error={}",
                request.connection_id, request.table_name, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let table_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_table(
            request.table_name.clone(),
            table,
            request.connection_id.clone(),
        ),
        Err(_) => {
            error!("open_table_v1 failed to lock table manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock table manager");
        }
    };

    info!(
        "open_table_v1 ok connection_id={} table_id={} table=\"{}\" elapsed_ms={}",
        request.connection_id,
        table_id,
        request.table_name,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(TableHandle {
        table_id,
        name: request.table_name,
    })
}

pub async fn get_schema_v1(
    state: &AppState,
    request: GetSchemaRequestV1,
) -> ResultEnvelope<SchemaDefinition> {
    let started_at = Instant::now();
    info!("get_schema_v1 start table_id={}", request.table_id);
    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("get_schema_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("get_schema_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let schema = match table.schema().await {
        Ok(schema) => schema,
        Err(error) => {
            error!("get_schema_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let definition = SchemaDefinition::from_arrow_schema(schema.as_ref());
    info!(
        "get_schema_v1 ok table_id={} fields={} elapsed_ms={}",
        request.table_id,
        definition.fields.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(definition)
}

pub async fn list_versions_v1(
    state: &AppState,
    request: ListVersionsRequestV1,
) -> ResultEnvelope<ListVersionsResponseV1> {
    let started_at = Instant::now();
    info!("list_versions_v1 start table_id={}", request.table_id);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("list_versions_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("list_versions_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let versions = match table.list_versions().await {
        Ok(versions) => versions.into_iter().map(to_version_info).collect::<Vec<_>>(),
        Err(error) => {
            error!("list_versions_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "list_versions_v1 ok table_id={} versions={} elapsed_ms={}",
        request.table_id,
        versions.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(ListVersionsResponseV1 { versions })
}

pub async fn get_table_version_v1(
    state: &AppState,
    request: GetTableVersionRequestV1,
) -> ResultEnvelope<GetTableVersionResponseV1> {
    let started_at = Instant::now();
    info!("get_table_version_v1 start table_id={}", request.table_id);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("get_table_version_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("get_table_version_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let version = match table.version().await {
        Ok(version) => version,
        Err(error) => {
            error!("get_table_version_v1 failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "get_table_version_v1 ok table_id={} version={} elapsed_ms={}",
        request.table_id,
        version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(GetTableVersionResponseV1 {
        table_id: request.table_id,
        version,
    })
}

pub async fn checkout_table_version_v1(
    state: &AppState,
    request: CheckoutTableVersionRequestV1,
) -> ResultEnvelope<CheckoutTableVersionResponseV1> {
    let started_at = Instant::now();
    info!(
        "checkout_table_version_v1 start table_id={} version={}",
        request.table_id, request.version
    );

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("checkout_table_version_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("checkout_table_version_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    if let Err(error) = table.checkout(request.version).await {
        error!(
            "checkout_table_version_v1 failed table_id={} error={}",
            request.table_id, error
        );
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    let version = match table.version().await {
        Ok(version) => version,
        Err(error) => {
            error!("checkout_table_version_v1 read version failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "checkout_table_version_v1 ok table_id={} version={} elapsed_ms={}",
        request.table_id,
        version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(CheckoutTableVersionResponseV1 {
        table_id: request.table_id,
        version,
    })
}

pub async fn checkout_table_latest_v1(
    state: &AppState,
    request: CheckoutTableLatestRequestV1,
) -> ResultEnvelope<CheckoutTableLatestResponseV1> {
    let started_at = Instant::now();
    info!("checkout_table_latest_v1 start table_id={}", request.table_id);

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("checkout_table_latest_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("checkout_table_latest_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    if let Err(error) = table.checkout_latest().await {
        error!(
            "checkout_table_latest_v1 failed table_id={} error={}",
            request.table_id, error
        );
        return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
    }

    let version = match table.version().await {
        Ok(version) => version,
        Err(error) => {
            error!("checkout_table_latest_v1 read version failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    info!(
        "checkout_table_latest_v1 ok table_id={} version={} elapsed_ms={}",
        request.table_id,
        version,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(CheckoutTableLatestResponseV1 {
        table_id: request.table_id,
        version,
    })
}

pub async fn clone_table_v1(
    state: &AppState,
    request: CloneTableRequestV1,
) -> ResultEnvelope<CloneTableResponseV1> {
    let started_at = Instant::now();
    info!(
        "clone_table_v1 start connection_id={} table_id={} target=\"{}\"",
        request.connection_id,
        request.table_id,
        request.target_table_name
    );

    let target_name = request.target_table_name.trim();
    if target_name.is_empty() {
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "target table name cannot be empty");
    }

    let (connection, table) = match state.connections.lock() {
        Ok(manager) => {
            let connection = manager.get_connection(&request.connection_id);
            let table = manager.get_table(&request.table_id);
            (connection, table)
        }
        Err(_) => {
            error!("clone_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(connection) = connection else {
        warn!("clone_table_v1 connection not found connection_id={}", request.connection_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "connection not found");
    };

    let Some(table) = table else {
        warn!("clone_table_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let source_uri = table.dataset_uri().to_string();
    let mut builder = connection.clone_table(target_name.to_string(), source_uri);
    if let Some(version) = request.source_version {
        builder = builder.source_version(version);
    }
    if let Some(tag) = request
        .source_tag
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        builder = builder.source_tag(tag.to_string());
    }
    if let Some(is_shallow) = request.is_shallow {
        builder = builder.is_shallow(is_shallow);
    }

    let cloned = match builder.execute().await {
        Ok(table) => table,
        Err(error) => {
            error!("clone_table_v1 failed error={}", error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let table_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_table(
            target_name.to_string(),
            cloned,
            request.connection_id.clone(),
        ),
        Err(_) => {
            error!("clone_table_v1 failed to lock table manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock table manager");
        }
    };

    info!(
        "clone_table_v1 ok table_id={} name=\"{}\" elapsed_ms={}",
        table_id,
        target_name,
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(CloneTableResponseV1 {
        table_id,
        name: target_name.to_string(),
    })
}

pub async fn scan_v1(state: &AppState, request: ScanRequestV1) -> ResultEnvelope<ScanResponseV1> {
    let started_at = Instant::now();
    info!(
        "scan_v1 start table_id={} format={:?} limit={:?} offset={:?}",
        request.table_id, request.format, request.limit, request.offset
    );
    if let Some(ref filter) = request.filter {
        trace!("scan_v1 filter=\"{}\"", filter);
    }
    if let Some(ref projection) = request.projection {
        trace!("scan_v1 projection={:?}", projection);
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("scan_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("scan_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let limit = request.limit.unwrap_or(100);
    let offset = request.offset.unwrap_or(0);
    let projection = request.projection.clone();
    let filter = request.filter.clone();
    let query_limit = limit.saturating_add(1);

    let fallback_schema = match table.schema().await {
        Ok(schema) => schema,
        Err(error) => {
            error!("scan_v1 failed to read schema table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let options = QueryOptions {
        projection,
        filter,
        limit: Some(query_limit),
        offset: Some(offset),
    };

    let query = apply_query_options(table.query(), &options);

    match request.format {
        DataFormat::Json => {
            let fallback_definition = SchemaDefinition::from_arrow_schema(fallback_schema.as_ref());
            let (mut rows, schema) = match execute_query_json(query, fallback_definition).await {
                Ok(result) => result,
                Err(error) => {
                    error!("scan_v1 query failed table_id={} error={}", request.table_id, error);
                    return ResultEnvelope::err(ErrorCode::Internal, error);
                }
            };

            let has_more = rows.len() > limit;
            if has_more {
                rows.truncate(limit);
            }
            let next_offset = if has_more {
                Some(offset.saturating_add(limit))
            } else {
                None
            };

            info!(
                "scan_v1 ok table_id={} rows={} next_offset={:?} elapsed_ms={}",
                request.table_id,
                rows.len(),
                next_offset,
                started_at.elapsed().as_millis()
            );

            ResultEnvelope::ok(ScanResponseV1 {
                chunk: DataChunk::Json(JsonChunk {
                    rows,
                    schema,
                    offset,
                    limit,
                }),
                next_offset,
            })
        }
        DataFormat::Arrow => {
            let batches = match execute_query_batches(query).await {
                Ok(result) => result,
                Err(error) => {
                    error!("scan_v1 query failed table_id={} error={}", request.table_id, error);
                    return ResultEnvelope::err(ErrorCode::Internal, error);
                }
            };

            let output_schema = batches
                .first()
                .map(|batch| batch.schema())
                .unwrap_or_else(|| fallback_schema.clone());
            let total_rows: usize = batches.iter().map(|batch| batch.num_rows()).sum();
            let has_more = total_rows > limit;
            let trimmed = if has_more {
                truncate_batches(&batches, limit)
            } else {
                batches
            };

            let ipc_base64 = match batches_to_arrow_ipc_base64(&trimmed, output_schema.as_ref()) {
                Ok(payload) => payload,
                Err(error) => {
                    error!(
                        "scan_v1 arrow encode failed table_id={} error={}",
                        request.table_id, error
                    );
                    return ResultEnvelope::err(ErrorCode::Internal, error);
                }
            };

            let next_offset = if has_more {
                Some(offset.saturating_add(limit))
            } else {
                None
            };

            info!(
                "scan_v1 ok arrow table_id={} rows={} next_offset={:?} elapsed_ms={}",
                request.table_id,
                total_rows.min(limit),
                next_offset,
                started_at.elapsed().as_millis()
            );

            ResultEnvelope::ok(ScanResponseV1 {
                chunk: DataChunk::Arrow(ArrowChunk {
                    ipc_base64,
                    compression: None,
                }),
                next_offset,
            })
        }
    }
}

pub async fn query_filter_v1(
    state: &AppState,
    request: QueryFilterRequestV1,
) -> ResultEnvelope<QueryResponseV1> {
    let started_at = Instant::now();
    info!(
        "query_filter_v1 start table_id={} limit={:?} offset={:?}",
        request.table_id, request.limit, request.offset
    );
    trace!("query_filter_v1 filter=\"{}\"", request.filter);
    if let Some(ref projection) = request.projection {
        trace!("query_filter_v1 projection={:?}", projection);
    }

    if request.filter.trim().is_empty() {
        warn!("query_filter_v1 empty filter table_id={}", request.table_id);
        return ResultEnvelope::err(
            ErrorCode::InvalidArgument,
            "filter expression cannot be empty",
        );
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("query_filter_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("query_filter_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let fallback_schema = match table.schema().await {
        Ok(schema) => SchemaDefinition::from_arrow_schema(schema.as_ref()),
        Err(error) => {
            error!(
                "query_filter_v1 failed to read schema table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let limit = request.limit.unwrap_or(100);
    let offset = request.offset.unwrap_or(0);
    let query_limit = limit.saturating_add(1);
    let options = QueryOptions {
        projection: request.projection,
        filter: Some(request.filter),
        limit: Some(query_limit),
        offset: Some(offset),
    };

    let query = apply_query_options(table.query(), &options);
    let (mut rows, schema) = match execute_query_json(query, fallback_schema).await {
        Ok(result) => result,
        Err(error) => {
            error!(
                "query_filter_v1 query failed table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };

    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }
    let next_offset = if has_more {
        Some(offset.saturating_add(limit))
    } else {
        None
    };

    info!(
        "query_filter_v1 ok table_id={} rows={} elapsed_ms={}",
        request.table_id,
        rows.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(QueryResponseV1 {
        chunk: DataChunk::Json(JsonChunk {
            rows,
            schema,
            offset,
            limit,
        }),
        next_offset,
    })
}

pub async fn combined_search_v1(
    state: &AppState,
    request: CombinedSearchRequestV1,
) -> ResultEnvelope<QueryResponseV1> {
    let started_at = Instant::now();
    info!(
        "combined_search_v1 start table_id={} limit={:?} offset={:?}",
        request.table_id, request.limit, request.offset
    );

    let has_vector = request
        .vector
        .as_ref()
        .map(|vector| !vector.is_empty())
        .unwrap_or(false);
    let query_text = request
        .query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if !has_vector && query_text.is_none() {
        warn!("combined_search_v1 missing vector and query table_id={}", request.table_id);
        return ResultEnvelope::err(
            ErrorCode::InvalidArgument,
            "vector or query text is required",
        );
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("combined_search_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("combined_search_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let fallback_schema = match table.schema().await {
        Ok(schema) => SchemaDefinition::from_arrow_schema(schema.as_ref()),
        Err(error) => {
            error!(
                "combined_search_v1 failed to read schema table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let limit = request.limit.unwrap_or(50);
    let offset = request.offset.unwrap_or(0);
    let fetch_limit = limit.saturating_add(offset);
    let projection = request
        .projection
        .as_ref()
        .filter(|value| !value.is_empty())
        .cloned();
    let filter = request.filter.as_ref().and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    let mut merged_rows: Vec<serde_json::Value> = Vec::new();
    let mut seen = HashSet::new();
    let mut result_schema: Option<SchemaDefinition> = None;

    if has_vector {
        let vector = request.vector.clone().unwrap_or_default();
        let mut vector_query = match table.query().nearest_to(vector) {
            Ok(query) => query,
            Err(error) => {
                error!(
                    "combined_search_v1 invalid vector query table_id={} error={}",
                    request.table_id, error
                );
                return ResultEnvelope::err(ErrorCode::InvalidArgument, error.to_string());
            }
        };

        if let Some(column) = request
            .vector_column
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            vector_query = vector_query.column(column);
        }
        if let Some(nprobes) = request.nprobes {
            vector_query = vector_query.nprobes(nprobes);
        }
        if let Some(refine_factor) = request.refine_factor {
            vector_query = vector_query.refine_factor(refine_factor);
        }

        let options = QueryOptions {
            projection: projection.clone(),
            filter: filter.clone(),
            limit: Some(fetch_limit),
            offset: None,
        };

        let query = apply_query_options(vector_query, &options);
        let (rows, schema) = match execute_query_json(query, fallback_schema.clone()).await {
            Ok(result) => result,
            Err(error) => {
                error!(
                    "combined_search_v1 vector query failed table_id={} error={}",
                    request.table_id, error
                );
                return ResultEnvelope::err(ErrorCode::Internal, error);
            }
        };
        result_schema = Some(schema);
        for row in rows {
            let key = serde_json::to_string(&row).unwrap_or_default();
            if seen.insert(key) {
                merged_rows.push(row);
            }
        }
    }

    if let Some(query_text) = query_text {
        let mut fts_query = FullTextSearchQuery::new(query_text.to_string());
        if let Some(columns) = request.columns.as_ref() {
            if !columns.is_empty() {
                fts_query = match fts_query.with_columns(columns) {
                    Ok(query) => query,
                    Err(error) => {
                        error!(
                            "combined_search_v1 invalid columns table_id={} error={}",
                            request.table_id, error
                        );
                        return ResultEnvelope::err(ErrorCode::InvalidArgument, error.to_string());
                    }
                };
            }
        }

        let options = QueryOptions {
            projection: projection.clone(),
            filter: filter.clone(),
            limit: Some(fetch_limit),
            offset: None,
        };

        let query = apply_query_options(table.query().full_text_search(fts_query), &options);
        let (rows, schema) = match execute_query_json(query, fallback_schema.clone()).await {
            Ok(result) => result,
            Err(error) => {
                error!(
                    "combined_search_v1 fts query failed table_id={} error={}",
                    request.table_id, error
                );
                return ResultEnvelope::err(ErrorCode::Internal, error);
            }
        };
        if result_schema.is_none() {
            result_schema = Some(schema);
        }
        for row in rows {
            let key = serde_json::to_string(&row).unwrap_or_default();
            if seen.insert(key) {
                merged_rows.push(row);
            }
        }
    }

    let total_rows = merged_rows.len();
    let next_offset = if total_rows > offset.saturating_add(limit) {
        Some(offset.saturating_add(limit))
    } else {
        None
    };
    let rows = merged_rows
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let schema = result_schema.unwrap_or(fallback_schema);

    info!(
        "combined_search_v1 ok table_id={} rows={} elapsed_ms={}",
        request.table_id,
        rows.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(QueryResponseV1 {
        chunk: DataChunk::Json(JsonChunk {
            rows,
            schema,
            offset,
            limit,
        }),
        next_offset,
    })
}

pub async fn vector_search_v1(
    state: &AppState,
    request: VectorSearchRequestV1,
) -> ResultEnvelope<QueryResponseV1> {
    let started_at = Instant::now();
    info!(
        "vector_search_v1 start table_id={} vector_len={} top_k={:?} offset={:?}",
        request.table_id,
        request.vector.len(),
        request.top_k,
        request.offset
    );
    if let Some(ref column) = request.column {
        trace!("vector_search_v1 column=\"{}\"", column);
    }
    if let Some(ref projection) = request.projection {
        trace!("vector_search_v1 projection={:?}", projection);
    }
    if let Some(ref filter) = request.filter {
        trace!("vector_search_v1 filter=\"{}\"", filter);
    }
    if let Some(nprobes) = request.nprobes {
        trace!("vector_search_v1 nprobes={}", nprobes);
    }
    if let Some(refine_factor) = request.refine_factor {
        trace!("vector_search_v1 refine_factor={}", refine_factor);
    }

    if request.vector.is_empty() {
        warn!("vector_search_v1 empty vector table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "vector must not be empty");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("vector_search_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("vector_search_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let fallback_schema = match table.schema().await {
        Ok(schema) => SchemaDefinition::from_arrow_schema(schema.as_ref()),
        Err(error) => {
            error!(
                "vector_search_v1 failed to read schema table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let mut vector_query = match table.query().nearest_to(request.vector) {
        Ok(query) => query,
        Err(error) => {
            error!(
                "vector_search_v1 invalid vector query table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::InvalidArgument, error.to_string());
        }
    };

    if let Some(column) = request.column.as_deref() {
        vector_query = vector_query.column(column);
    }

    if let Some(nprobes) = request.nprobes {
        vector_query = vector_query.nprobes(nprobes);
    }

    if let Some(refine_factor) = request.refine_factor {
        vector_query = vector_query.refine_factor(refine_factor);
    }

    let limit = request.top_k.unwrap_or(10);
    let offset = request.offset.unwrap_or(0);
    let query_limit = limit.saturating_add(1);
    let options = QueryOptions {
        projection: request.projection,
        filter: request.filter,
        limit: Some(query_limit),
        offset: Some(offset),
    };

    let query = apply_query_options(vector_query, &options);
    let (mut rows, schema) = match execute_query_json(query, fallback_schema).await {
        Ok(result) => result,
        Err(error) => {
            error!("vector_search_v1 query failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };

    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }
    let next_offset = if has_more {
        Some(offset.saturating_add(limit))
    } else {
        None
    };

    info!(
        "vector_search_v1 ok table_id={} rows={} elapsed_ms={}",
        request.table_id,
        rows.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(QueryResponseV1 {
        chunk: DataChunk::Json(JsonChunk {
            rows,
            schema,
            offset,
            limit,
        }),
        next_offset,
    })
}

pub async fn fts_search_v1(
    state: &AppState,
    request: FtsSearchRequestV1,
) -> ResultEnvelope<QueryResponseV1> {
    let started_at = Instant::now();
    info!(
        "fts_search_v1 start table_id={} limit={:?} offset={:?}",
        request.table_id, request.limit, request.offset
    );
    trace!("fts_search_v1 query=\"{}\"", request.query);
    if let Some(ref columns) = request.columns {
        trace!("fts_search_v1 columns={:?}", columns);
    }
    if let Some(ref projection) = request.projection {
        trace!("fts_search_v1 projection={:?}", projection);
    }
    if let Some(ref filter) = request.filter {
        trace!("fts_search_v1 filter=\"{}\"", filter);
    }

    if request.query.trim().is_empty() {
        warn!("fts_search_v1 empty query table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::InvalidArgument, "query text cannot be empty");
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("fts_search_v1 failed to lock connection manager");
            return ResultEnvelope::err(ErrorCode::Internal, "failed to lock connection manager");
        }
    };

    let Some(table) = table else {
        warn!("fts_search_v1 table not found table_id={}", request.table_id);
        return ResultEnvelope::err(ErrorCode::NotFound, "table not found");
    };

    let fallback_schema = match table.schema().await {
        Ok(schema) => SchemaDefinition::from_arrow_schema(schema.as_ref()),
        Err(error) => {
            error!(
                "fts_search_v1 failed to read schema table_id={} error={}",
                request.table_id, error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let mut fts_query = FullTextSearchQuery::new(request.query);
    if let Some(columns) = request.columns {
        if !columns.is_empty() {
            fts_query = match fts_query.with_columns(&columns) {
                Ok(query) => query,
                Err(error) => {
                    error!(
                        "fts_search_v1 invalid columns table_id={} error={}",
                        request.table_id, error
                    );
                    return ResultEnvelope::err(ErrorCode::InvalidArgument, error.to_string());
                }
            };
        }
    }

    let limit = request.limit.unwrap_or(100);
    let offset = request.offset.unwrap_or(0);
    let query_limit = limit.saturating_add(1);
    let options = QueryOptions {
        projection: request.projection,
        filter: request.filter,
        limit: Some(query_limit),
        offset: Some(offset),
    };

    let query = apply_query_options(table.query().full_text_search(fts_query), &options);
    let (mut rows, schema) = match execute_query_json(query, fallback_schema).await {
        Ok(result) => result,
        Err(error) => {
            error!("fts_search_v1 query failed table_id={} error={}", request.table_id, error);
            return ResultEnvelope::err(ErrorCode::Internal, error);
        }
    };

    let has_more = rows.len() > limit;
    if has_more {
        rows.truncate(limit);
    }
    let next_offset = if has_more {
        Some(offset.saturating_add(limit))
    } else {
        None
    };

    info!(
        "fts_search_v1 ok table_id={} rows={} elapsed_ms={}",
        request.table_id,
        rows.len(),
        started_at.elapsed().as_millis()
    );

    ResultEnvelope::ok(QueryResponseV1 {
        chunk: DataChunk::Json(JsonChunk {
            rows,
            schema,
            offset,
            limit,
        }),
        next_offset,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use arrow_array::Int32Array;
    use arrow_schema::{DataType, Field, Schema};

    use super::truncate_batches;

    fn make_batch(values: &[i32]) -> arrow_array::RecordBatch {
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int32, false)]));
        let array = Int32Array::from_iter_values(values.iter().copied());
        arrow_array::RecordBatch::try_new(schema, vec![Arc::new(array)])
            .expect("create record batch")
    }

    #[test]
    fn truncate_batches_respects_limit() {
        let batch1 = make_batch(&[1, 2]);
        let batch2 = make_batch(&[3, 4]);

        let trimmed = truncate_batches(&[batch1, batch2], 3);
        let total_rows: usize = trimmed.iter().map(|batch| batch.num_rows()).sum();

        assert_eq!(trimmed.len(), 2);
        assert_eq!(total_rows, 3);
        assert_eq!(trimmed[1].num_rows(), 1);
    }
}
