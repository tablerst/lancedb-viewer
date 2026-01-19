use std::time::Duration;

use arrow_array::RecordBatch;
use arrow_json::ArrayWriter;
use futures_util::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};

use crate::domain::connect::infer_backend_kind;
use crate::ipc::v1::{
    ConnectRequestV1, ConnectResponseV1, DataChunk, DataFormat, ErrorCode, GetSchemaRequestV1,
    JsonChunk, ListTablesRequestV1, ListTablesResponseV1, OpenTableRequestV1, ResultEnvelope,
    ScanRequestV1, ScanResponseV1, SchemaDefinition, TableHandle, TableInfo,
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

#[tauri::command]
pub async fn connect_v1(
    state: tauri::State<'_, AppState>,
    request: ConnectRequestV1,
) -> Result<ResultEnvelope<ConnectResponseV1>, String> {
    let profile = request.profile;
    let backend_kind = infer_backend_kind(&profile.uri);

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
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    let connection_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_connection(connection),
        Err(_) => {
            return Ok(ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            ))
        }
    };

    Ok(ResultEnvelope::ok(ConnectResponseV1 {
        connection_id,
        backend_kind,
        name: profile.name,
        uri: profile.uri,
    }))
}

#[tauri::command]
pub async fn list_tables_v1(
    state: tauri::State<'_, AppState>,
    request: ListTablesRequestV1,
) -> Result<ResultEnvelope<ListTablesResponseV1>, String> {
    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            return Ok(ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            ))
        }
    };

    let Some(connection) = connection else {
        return Ok(ResultEnvelope::err(ErrorCode::NotFound, "connection not found"));
    };

    let names = match connection.table_names().execute().await {
        Ok(names) => names,
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    let tables = names.into_iter().map(|name| TableInfo { name }).collect();

    Ok(ResultEnvelope::ok(ListTablesResponseV1 { tables }))
}

#[tauri::command]
pub async fn open_table_v1(
    state: tauri::State<'_, AppState>,
    request: OpenTableRequestV1,
) -> Result<ResultEnvelope<TableHandle>, String> {
    let connection = match state.connections.lock() {
        Ok(manager) => manager.get_connection(&request.connection_id),
        Err(_) => {
            return Ok(ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            ))
        }
    };

    let Some(connection) = connection else {
        return Ok(ResultEnvelope::err(ErrorCode::NotFound, "connection not found"));
    };

    let table = match connection.open_table(&request.table_name).execute().await {
        Ok(table) => table,
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    let table_id = match state.connections.lock() {
        Ok(mut manager) => manager.insert_table(request.table_name.clone(), table),
        Err(_) => {
            return Ok(ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock table manager",
            ))
        }
    };

    Ok(ResultEnvelope::ok(TableHandle {
        table_id,
        name: request.table_name,
    }))
}

#[tauri::command]
pub async fn get_schema_v1(
    state: tauri::State<'_, AppState>,
    request: GetSchemaRequestV1,
) -> Result<ResultEnvelope<SchemaDefinition>, String> {
    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            return Ok(ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            ))
        }
    };

    let Some(table) = table else {
        return Ok(ResultEnvelope::err(ErrorCode::NotFound, "table not found"));
    };

    let schema = match table.schema().await {
        Ok(schema) => schema,
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    Ok(ResultEnvelope::ok(SchemaDefinition::from_arrow_schema(
        schema.as_ref(),
    )))
}

#[tauri::command]
pub async fn scan_v1(
    state: tauri::State<'_, AppState>,
    request: ScanRequestV1,
) -> Result<ResultEnvelope<ScanResponseV1>, String> {
    if matches!(request.format, DataFormat::Arrow) {
        return Ok(ResultEnvelope::err(
            ErrorCode::NotImplemented,
            "arrow format is not implemented yet",
        ));
    }

    let table = match state.connections.lock() {
        Ok(manager) => manager.get_table(&request.table_id),
        Err(_) => {
            return Ok(ResultEnvelope::err(
                ErrorCode::Internal,
                "failed to lock connection manager",
            ))
        }
    };

    let Some(table) = table else {
        return Ok(ResultEnvelope::err(ErrorCode::NotFound, "table not found"));
    };

    let limit = request.limit.unwrap_or(100);
    let offset = request.offset.unwrap_or(0);
    let mut query = table.query();

    if let Some(filter) = request.filter.as_deref() {
        query = query.only_if(filter);
    }

    let query_limit = limit.saturating_add(offset);
    query = query.limit(query_limit);

    let stream = match query.execute().await {
        Ok(stream) => stream,
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    let batches = match stream.try_collect::<Vec<_>>().await {
        Ok(batches) => batches,
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    let schema = if let Some(first) = batches.first() {
        SchemaDefinition::from_arrow_schema(first.schema().as_ref())
    } else {
        let schema = match table.schema().await {
            Ok(schema) => schema,
            Err(error) => {
                return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string()))
            }
        };
        SchemaDefinition::from_arrow_schema(schema.as_ref())
    };

    let mut rows = match batches_to_json_rows(&batches) {
        Ok(rows) => rows,
        Err(error) => return Ok(ResultEnvelope::err(ErrorCode::Internal, error.to_string())),
    };

    let total = rows.len();
    let start = offset.min(total);
    let end = offset.saturating_add(limit).min(total);
    rows = rows[start..end].to_vec();

    let next_offset = if end < total { Some(end) } else { None };

    Ok(ResultEnvelope::ok(ScanResponseV1 {
        chunk: DataChunk::Json(JsonChunk {
            rows,
            schema,
            offset,
            limit,
        }),
        next_offset,
    }))
}
