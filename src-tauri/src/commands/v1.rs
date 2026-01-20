use std::time::{Duration, Instant};

use arrow_array::RecordBatch;
use arrow_ipc::writer::StreamWriter;
use arrow_json::ArrayWriter;
use arrow_schema::Schema;
use base64::{engine::general_purpose, Engine as _};
use futures_util::TryStreamExt;
use lancedb::index::scalar::FullTextSearchQuery;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::query::Select;
use log::{debug, error, info, trace, warn};

use crate::domain::connect::infer_backend_kind;
use crate::ipc::v1::{
    ConnectRequestV1, ConnectResponseV1, DataChunk, DataFormat, ErrorCode, FtsSearchRequestV1,
    GetSchemaRequestV1, JsonChunk, ListTablesRequestV1, ListTablesResponseV1, OpenTableRequestV1,
    QueryFilterRequestV1, QueryResponseV1, ResultEnvelope, ScanRequestV1, ScanResponseV1,
    SchemaDefinition, TableHandle, TableInfo, VectorSearchRequestV1,
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

fn batches_to_arrow_ipc_base64(
    batches: &[RecordBatch],
    schema: &Schema,
) -> Result<String, String> {
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

async fn connect_v1_impl(
    state: &AppState,
    request: ConnectRequestV1,
) -> ResultEnvelope<ConnectResponseV1> {
    let started_at = Instant::now();
    let profile = request.profile;
    let backend_kind = infer_backend_kind(&profile.uri);

    info!(
        "connect_v1 start name=\"{}\" uri=\"{}\" backend={:?}",
        profile.name,
        profile.uri,
        backend_kind
    );
    if !profile.storage_options.is_empty() {
        let keys: Vec<String> = profile.storage_options.keys().cloned().collect();
        trace!("connect_v1 storage_options_keys={:?}", keys);
    }
    if let Some(interval) = profile.options.read_consistency_interval_seconds {
        debug!("connect_v1 read_consistency_interval_seconds={}", interval);
    }

    let mut builder = lancedb::connect(&profile.uri);
    if !profile.storage_options.is_empty() {
        builder = builder.storage_options(
            profile
                .storage_options
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
                profile.uri,
                error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let connection_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_connection(connection),
        Err(_) => {
            error!("connect_v1 failed to lock connection manager");
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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

#[tauri::command]
pub async fn connect_v1(
    state: tauri::State<'_, AppState>,
    request: ConnectRequestV1,
) -> Result<ResultEnvelope<ConnectResponseV1>, String> {
    Ok(connect_v1_impl(state.inner(), request).await)
}

async fn list_tables_v1_impl(
    state: &AppState,
    request: ListTablesRequestV1,
) -> ResultEnvelope<ListTablesResponseV1> {
    let started_at = Instant::now();
    info!("list_tables_v1 start connection_id={}", request.connection_id);
    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("list_tables_v1 failed to lock connection manager");
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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
                request.connection_id,
                error
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

#[tauri::command]
pub async fn list_tables_v1(
    state: tauri::State<'_, AppState>,
    request: ListTablesRequestV1,
) -> Result<ResultEnvelope<ListTablesResponseV1>, String> {
    Ok(list_tables_v1_impl(state.inner(), request).await)
}

async fn open_table_v1_impl(
    state: &AppState,
    request: OpenTableRequestV1,
) -> ResultEnvelope<TableHandle> {
    let started_at = Instant::now();
    info!(
        "open_table_v1 start connection_id={} table=\"{}\"",
        request.connection_id,
        request.table_name
    );
    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            error!("open_table_v1 failed to lock connection manager");
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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
                request.connection_id,
                request.table_name,
                error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let table_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_table(request.table_name.clone(), table),
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

#[tauri::command]
pub async fn open_table_v1(
    state: tauri::State<'_, AppState>,
    request: OpenTableRequestV1,
) -> Result<ResultEnvelope<TableHandle>, String> {
    Ok(open_table_v1_impl(state.inner(), request).await)
}

async fn get_schema_v1_impl(
    state: &AppState,
    request: GetSchemaRequestV1,
) -> ResultEnvelope<SchemaDefinition> {
    let started_at = Instant::now();
    info!("get_schema_v1 start table_id={}", request.table_id);
    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("get_schema_v1 failed to lock connection manager");
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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

#[tauri::command]
pub async fn get_schema_v1(
    state: tauri::State<'_, AppState>,
    request: GetSchemaRequestV1,
) -> Result<ResultEnvelope<SchemaDefinition>, String> {
    Ok(get_schema_v1_impl(state.inner(), request).await)
}

async fn scan_v1_impl(state: &AppState, request: ScanRequestV1) -> ResultEnvelope<ScanResponseV1> {
    let started_at = Instant::now();
    info!(
        "scan_v1 start table_id={} format={:?} limit={:?} offset={:?}",
        request.table_id,
        request.format,
        request.limit,
        request.offset
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
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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
            let fallback_definition =
                SchemaDefinition::from_arrow_schema(fallback_schema.as_ref());
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
                        request.table_id,
                        error
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
                chunk: DataChunk::Arrow(crate::ipc::v1::ArrowChunk {
                    ipc_base64,
                    compression: None,
                }),
                next_offset,
            })
        }
    }
}

#[tauri::command]
pub async fn scan_v1(
    state: tauri::State<'_, AppState>,
    request: ScanRequestV1,
) -> Result<ResultEnvelope<ScanResponseV1>, String> {
    Ok(scan_v1_impl(state.inner(), request).await)
}

#[tauri::command]
pub async fn query_filter_v1(
    state: tauri::State<'_, AppState>,
    request: QueryFilterRequestV1,
) -> Result<ResultEnvelope<QueryResponseV1>, String> {
    Ok(query_filter_v1_impl(state.inner(), request).await)
}

async fn query_filter_v1_impl(
    state: &AppState,
    request: QueryFilterRequestV1,
) -> ResultEnvelope<QueryResponseV1> {
    let started_at = Instant::now();
    info!(
        "query_filter_v1 start table_id={} limit={:?} offset={:?}",
        request.table_id,
        request.limit,
        request.offset
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
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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
                request.table_id,
                error
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
                request.table_id,
                error
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

#[tauri::command]
pub async fn vector_search_v1(
    state: tauri::State<'_, AppState>,
    request: VectorSearchRequestV1,
) -> Result<ResultEnvelope<QueryResponseV1>, String> {
    Ok(vector_search_v1_impl(state.inner(), request).await)
}

async fn vector_search_v1_impl(
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
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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
                request.table_id,
                error
            );
            return ResultEnvelope::err(ErrorCode::Internal, error.to_string());
        }
    };

    let mut vector_query = match table.query().nearest_to(request.vector) {
        Ok(query) => query,
        Err(error) => {
            error!(
                "vector_search_v1 invalid vector query table_id={} error={}",
                request.table_id,
                error
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

#[tauri::command]
pub async fn fts_search_v1(
    state: tauri::State<'_, AppState>,
    request: FtsSearchRequestV1,
) -> Result<ResultEnvelope<QueryResponseV1>, String> {
    Ok(fts_search_v1_impl(state.inner(), request).await)
}

async fn fts_search_v1_impl(
    state: &AppState,
    request: FtsSearchRequestV1,
) -> ResultEnvelope<QueryResponseV1> {
    let started_at = Instant::now();
    info!(
        "fts_search_v1 start table_id={} limit={:?} offset={:?}",
        request.table_id,
        request.limit,
        request.offset
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
        return ResultEnvelope::err(
            ErrorCode::InvalidArgument,
            "query text cannot be empty",
        );
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            error!("fts_search_v1 failed to lock connection manager");
            return ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            );
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
                request.table_id,
                error
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
                        request.table_id,
                        error
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
    use std::{io::Cursor, panic};

    use arrow_array::types::Float32Type;
    use arrow_array::{FixedSizeListArray, Int32Array, RecordBatch, RecordBatchIterator, StringArray};
    use arrow_ipc::reader::StreamReader;
    use arrow_schema::{DataType, Field, Schema};
    use base64::{engine::general_purpose, Engine as _};
    use lancedb::index::Index;
    use tempfile::tempdir;

    use super::*;
    use crate::ipc::v1::ConnectProfile;

    struct TestTable {
        _dir: tempfile::TempDir,
        table: lancedb::Table,
    }

    async fn create_test_table() -> TestTable {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    3,
                ),
                false,
            ),
        ]));

        let ids = Int32Array::from_iter_values(0..5);
        let texts = StringArray::from(vec!["alpha", "beta", "gamma", "delta", "epsilon"]);
        let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            vec![
                Some(vec![Some(0.1), Some(0.2), Some(0.3)]),
                Some(vec![Some(0.0), Some(0.1), Some(0.0)]),
                Some(vec![Some(0.2), Some(0.1), Some(0.0)]),
                Some(vec![Some(0.9), Some(0.9), Some(0.9)]),
                Some(vec![Some(0.5), Some(0.4), Some(0.3)]),
            ],
            3,
        );

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(ids), Arc::new(texts), Arc::new(vectors)],
        )
        .expect("create record batch");

        let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);
        let dir = tempdir().expect("create tempdir");
        let db = lancedb::connect(dir.path().to_str().expect("tempdir path"))
            .execute()
            .await
            .expect("connect lancedb");
        let table = db
            .create_table("items", Box::new(batches))
            .execute()
            .await
            .expect("create table");

        TestTable { _dir: dir, table }
    }

    struct CommandHarness {
        _dir: tempfile::TempDir,
        state: AppState,
        connection_id: String,
        table_id: String,
    }

    async fn seed_items_table(connection: &lancedb::Connection) {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("text", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    3,
                ),
                false,
            ),
        ]));

        let ids = Int32Array::from_iter_values(0..5);
        let texts = StringArray::from(vec!["alpha", "beta", "gamma", "delta", "epsilon"]);
        let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            vec![
                Some(vec![Some(0.1), Some(0.2), Some(0.3)]),
                Some(vec![Some(0.0), Some(0.1), Some(0.0)]),
                Some(vec![Some(0.2), Some(0.1), Some(0.0)]),
                Some(vec![Some(0.9), Some(0.9), Some(0.9)]),
                Some(vec![Some(0.5), Some(0.4), Some(0.3)]),
            ],
            3,
        );

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(ids), Arc::new(texts), Arc::new(vectors)],
        )
        .expect("create record batch");

        let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);

        connection
            .create_table("items", Box::new(batches))
            .execute()
            .await
            .expect("create table");
    }

    async fn create_command_harness() -> CommandHarness {
        let dir = tempdir().expect("create tempdir");
        let uri = dir.path().to_str().expect("tempdir path").to_string();
        let state = AppState::new();

        let envelope = connect_v1_impl(
            &state,
            ConnectRequestV1 {
                profile: ConnectProfile {
                    name: "test".to_string(),
                    uri,
                    storage_options: Default::default(),
                    options: Default::default(),
                    auth: Default::default(),
                },
            },
        )
        .await;

        assert!(envelope.ok, "connect should succeed: {:?}", envelope.error);
        let connection_id = envelope
            .data
            .as_ref()
            .expect("connect data")
            .connection_id
            .clone();

        let connection = state
            .connections
            .lock()
            .expect("lock connections")
            .get_connection(&connection_id)
            .expect("connection exists");

        seed_items_table(&connection).await;

        let opened = open_table_v1_impl(
            &state,
            OpenTableRequestV1 {
                connection_id: connection_id.clone(),
                table_name: "items".to_string(),
            },
        )
        .await;
        assert!(opened.ok, "open_table should succeed: {:?}", opened.error);

        let table_id = opened.data.expect("table handle").table_id;

        CommandHarness {
            _dir: dir,
            state,
            connection_id,
            table_id,
        }
    }

    fn poison_connections_mutex(state: &AppState) {
        let result = panic::catch_unwind(|| {
            let _guard = state.connections.lock().expect("lock for poison");
            panic!("poison mutex");
        });
        assert!(result.is_err(), "expected a panic to poison the mutex");
    }

    #[tokio::test]
    async fn query_filter_returns_rows() {
        let test = create_test_table().await;
        let table = &test.table;
        let fallback_schema = SchemaDefinition::from_arrow_schema(
            table.schema().await.expect("schema").as_ref(),
        );
        let query = table.query().only_if("id >= 2").limit(2);
        let (rows, _) = execute_query_json(query, fallback_schema)
            .await
            .expect("execute query");
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn vector_search_returns_rows() {
        let test = create_test_table().await;
        let table = &test.table;
        let fallback_schema = SchemaDefinition::from_arrow_schema(
            table.schema().await.expect("schema").as_ref(),
        );
        let query = table
            .query()
            .nearest_to(vec![0.1_f32, 0.2, 0.3])
            .expect("nearest_to")
            .limit(2);
        let (rows, _) = execute_query_json(query, fallback_schema)
            .await
            .expect("execute query");
        assert_eq!(rows.len(), 2);
    }

    #[tokio::test]
    async fn fts_search_returns_rows() {
        let test = create_test_table().await;
        let table = &test.table;
        table
            .create_index(&["text"], Index::FTS(Default::default()))
            .execute()
            .await
            .expect("create fts index");

        let fallback_schema = SchemaDefinition::from_arrow_schema(
            table.schema().await.expect("schema").as_ref(),
        );
        let query = table
            .query()
            .full_text_search(FullTextSearchQuery::new("alpha".to_string()))
            .limit(5);
        let (rows, _) = execute_query_json(query, fallback_schema)
            .await
            .expect("execute query");
        assert!(!rows.is_empty());
    }

    #[tokio::test]
    async fn commands_happy_path_and_errors() {
        let harness = create_command_harness().await;

        let listed = list_tables_v1_impl(
            &harness.state,
            ListTablesRequestV1 {
                connection_id: harness.connection_id.clone(),
            },
        )
        .await;
        assert!(listed.ok);
        assert_eq!(listed.data.expect("tables").tables.len(), 1);

        let schema = get_schema_v1_impl(
            &harness.state,
            GetSchemaRequestV1 {
                table_id: harness.table_id.clone(),
            },
        )
        .await;
        assert!(schema.ok);
        assert!(
            schema.data.expect("schema").fields.iter().any(|f| f.name == "id"),
            "schema should contain id field"
        );

        // scan json pagination
        let scan_page1 = scan_v1_impl(
            &harness.state,
            ScanRequestV1 {
                table_id: harness.table_id.clone(),
                format: DataFormat::Json,
                projection: None,
                filter: None,
                limit: Some(2),
                offset: Some(0),
            },
        )
        .await;
        assert!(scan_page1.ok);
        let scan_page1 = scan_page1.data.expect("scan data");
        assert_eq!(scan_page1.next_offset, Some(2));
        match scan_page1.chunk {
            DataChunk::Json(chunk) => assert_eq!(chunk.rows.len(), 2),
            _ => panic!("expected json chunk"),
        }

        let scan_last = scan_v1_impl(
            &harness.state,
            ScanRequestV1 {
                table_id: harness.table_id.clone(),
                format: DataFormat::Json,
                projection: None,
                filter: None,
                limit: Some(2),
                offset: Some(4),
            },
        )
        .await;
        assert!(scan_last.ok);
        let scan_last = scan_last.data.expect("scan data");
        assert_eq!(scan_last.next_offset, None);
        match scan_last.chunk {
            DataChunk::Json(chunk) => assert_eq!(chunk.rows.len(), 1),
            _ => panic!("expected json chunk"),
        }

        // scan arrow
        let scan_arrow = scan_v1_impl(
            &harness.state,
            ScanRequestV1 {
                table_id: harness.table_id.clone(),
                format: DataFormat::Arrow,
                projection: None,
                filter: None,
                limit: Some(3),
                offset: Some(0),
            },
        )
        .await;
        assert!(scan_arrow.ok);
        let scan_arrow = scan_arrow.data.expect("scan arrow");
        assert_eq!(scan_arrow.next_offset, Some(3));

        let ipc_base64 = match scan_arrow.chunk {
            DataChunk::Arrow(chunk) => chunk.ipc_base64,
            _ => panic!("expected arrow chunk"),
        };

        let decoded = general_purpose::STANDARD
            .decode(ipc_base64)
            .expect("decode base64");
        let reader = StreamReader::try_new(Cursor::new(decoded), None).expect("open stream reader");
        let mut row_count = 0;
        for batch in reader {
            let batch = batch.expect("read batch");
            row_count += batch.num_rows();
        }
        assert_eq!(row_count, 3);

        // query filter
        let filtered = query_filter_v1_impl(
            &harness.state,
            QueryFilterRequestV1 {
                table_id: harness.table_id.clone(),
                filter: "id >= 2".to_string(),
                projection: None,
                limit: Some(2),
                offset: Some(0),
            },
        )
        .await;
        assert!(filtered.ok);

        let filtered_empty = query_filter_v1_impl(
            &harness.state,
            QueryFilterRequestV1 {
                table_id: harness.table_id.clone(),
                filter: "  ".to_string(),
                projection: None,
                limit: None,
                offset: None,
            },
        )
        .await;
        assert!(!filtered_empty.ok);
        assert_eq!(
            filtered_empty.error.expect("error").code,
            ErrorCode::InvalidArgument
        );

        // vector search
        let vector_ok = vector_search_v1_impl(
            &harness.state,
            VectorSearchRequestV1 {
                table_id: harness.table_id.clone(),
                vector: vec![0.1, 0.2, 0.3],
                column: None,
                top_k: Some(2),
                projection: None,
                filter: None,
                nprobes: None,
                refine_factor: None,
                offset: Some(0),
            },
        )
        .await;
        assert!(vector_ok.ok);

        let vector_empty = vector_search_v1_impl(
            &harness.state,
            VectorSearchRequestV1 {
                table_id: harness.table_id.clone(),
                vector: vec![],
                column: None,
                top_k: None,
                projection: None,
                filter: None,
                nprobes: None,
                refine_factor: None,
                offset: None,
            },
        )
        .await;
        assert!(!vector_empty.ok);
        assert_eq!(
            vector_empty.error.expect("error").code,
            ErrorCode::InvalidArgument
        );

        // fts search (needs index)
        let table = harness
            .state
            .connections
            .lock()
            .expect("lock")
            .get_table(&harness.table_id)
            .expect("table");
        table
            .create_index(&["text"], Index::FTS(Default::default()))
            .execute()
            .await
            .expect("create fts index");

        let fts_ok = fts_search_v1_impl(
            &harness.state,
            FtsSearchRequestV1 {
                table_id: harness.table_id.clone(),
                query: "alpha".to_string(),
                columns: None,
                limit: Some(10),
                offset: Some(0),
                projection: None,
                filter: None,
            },
        )
        .await;
        assert!(fts_ok.ok);

        let fts_empty = fts_search_v1_impl(
            &harness.state,
            FtsSearchRequestV1 {
                table_id: harness.table_id.clone(),
                query: " ".to_string(),
                columns: None,
                limit: None,
                offset: None,
                projection: None,
                filter: None,
            },
        )
        .await;
        assert!(!fts_empty.ok);
        assert_eq!(fts_empty.error.expect("error").code, ErrorCode::InvalidArgument);

        // not found
        let missing_schema = get_schema_v1_impl(
            &harness.state,
            GetSchemaRequestV1 {
                table_id: "missing".to_string(),
            },
        )
        .await;
        assert!(!missing_schema.ok);
        assert_eq!(missing_schema.error.expect("error").code, ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn commands_return_internal_when_mutex_poisoned() {
        let state = AppState::new();
        poison_connections_mutex(&state);

        let result = list_tables_v1_impl(
            &state,
            ListTablesRequestV1 {
                connection_id: "any".to_string(),
            },
        )
        .await;

        assert!(!result.ok);
        assert_eq!(result.error.expect("error").code, ErrorCode::Internal);
    }
}
