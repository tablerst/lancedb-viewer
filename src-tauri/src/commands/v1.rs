use crate::ipc::v1::{
    ConnectRequestV1, ConnectResponseV1, FtsSearchRequestV1, GetSchemaRequestV1,
    ListTablesRequestV1, ListTablesResponseV1, OpenTableRequestV1, QueryFilterRequestV1,
    QueryResponseV1, ResultEnvelope, ScanRequestV1, ScanResponseV1, SchemaDefinition, TableHandle,
    VectorSearchRequestV1,
};
use crate::services::v1 as services_v1;
use crate::state::AppState;

#[tauri::command]
pub async fn connect_v1(
    state: tauri::State<'_, AppState>,
    request: ConnectRequestV1,
) -> Result<ResultEnvelope<ConnectResponseV1>, String> {
    Ok(services_v1::connect_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn list_tables_v1(
    state: tauri::State<'_, AppState>,
    request: ListTablesRequestV1,
) -> Result<ResultEnvelope<ListTablesResponseV1>, String> {
    Ok(services_v1::list_tables_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn open_table_v1(
    state: tauri::State<'_, AppState>,
    request: OpenTableRequestV1,
) -> Result<ResultEnvelope<TableHandle>, String> {
    Ok(services_v1::open_table_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn get_schema_v1(
    state: tauri::State<'_, AppState>,
    request: GetSchemaRequestV1,
) -> Result<ResultEnvelope<SchemaDefinition>, String> {
    Ok(services_v1::get_schema_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn scan_v1(
    state: tauri::State<'_, AppState>,
    request: ScanRequestV1,
) -> Result<ResultEnvelope<ScanResponseV1>, String> {
    Ok(services_v1::scan_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn query_filter_v1(
    state: tauri::State<'_, AppState>,
    request: QueryFilterRequestV1,
) -> Result<ResultEnvelope<QueryResponseV1>, String> {
    Ok(services_v1::query_filter_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn vector_search_v1(
    state: tauri::State<'_, AppState>,
    request: VectorSearchRequestV1,
) -> Result<ResultEnvelope<QueryResponseV1>, String> {
    Ok(services_v1::vector_search_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn fts_search_v1(
    state: tauri::State<'_, AppState>,
    request: FtsSearchRequestV1,
) -> Result<ResultEnvelope<QueryResponseV1>, String> {
    Ok(services_v1::fts_search_v1(state.inner(), request).await)
}
