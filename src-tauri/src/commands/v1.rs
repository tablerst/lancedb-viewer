use crate::ipc::v1::{
    AddColumnsRequestV1, AddColumnsResponseV1, AlterColumnsRequestV1, AlterColumnsResponseV1,
    ConnectRequestV1, ConnectResponseV1, CreateIndexRequestV1, CreateIndexResponseV1,
    CreateTableRequestV1, CreateTableResponseV1, DeleteRowsRequestV1, DeleteRowsResponseV1,
    DropColumnsRequestV1, DropColumnsResponseV1, DropIndexRequestV1, DropIndexResponseV1,
    DropTableRequestV1, DropTableResponseV1, FtsSearchRequestV1, GetSchemaRequestV1,
    ListIndexesRequestV1, ListIndexesResponseV1, ListTablesRequestV1, ListTablesResponseV1,
    OpenTableRequestV1, QueryFilterRequestV1, QueryResponseV1, ResultEnvelope, ScanRequestV1,
    ScanResponseV1, SchemaDefinition, TableHandle, UpdateRowsRequestV1, UpdateRowsResponseV1,
    VectorSearchRequestV1, WriteRowsRequestV1, WriteRowsResponseV1,
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
pub async fn drop_table_v1(
    state: tauri::State<'_, AppState>,
    request: DropTableRequestV1,
) -> Result<ResultEnvelope<DropTableResponseV1>, String> {
    Ok(services_v1::drop_table_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn list_indexes_v1(
    state: tauri::State<'_, AppState>,
    request: ListIndexesRequestV1,
) -> Result<ResultEnvelope<ListIndexesResponseV1>, String> {
    Ok(services_v1::list_indexes_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn create_index_v1(
    state: tauri::State<'_, AppState>,
    request: CreateIndexRequestV1,
) -> Result<ResultEnvelope<CreateIndexResponseV1>, String> {
    Ok(services_v1::create_index_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn drop_index_v1(
    state: tauri::State<'_, AppState>,
    request: DropIndexRequestV1,
) -> Result<ResultEnvelope<DropIndexResponseV1>, String> {
    Ok(services_v1::drop_index_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn create_table_v1(
    state: tauri::State<'_, AppState>,
    request: CreateTableRequestV1,
) -> Result<ResultEnvelope<CreateTableResponseV1>, String> {
    Ok(services_v1::create_table_v1(state.inner(), request).await)
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
pub async fn add_columns_v1(
    state: tauri::State<'_, AppState>,
    request: AddColumnsRequestV1,
) -> Result<ResultEnvelope<AddColumnsResponseV1>, String> {
    Ok(services_v1::add_columns_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn alter_columns_v1(
    state: tauri::State<'_, AppState>,
    request: AlterColumnsRequestV1,
) -> Result<ResultEnvelope<AlterColumnsResponseV1>, String> {
    Ok(services_v1::alter_columns_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn drop_columns_v1(
    state: tauri::State<'_, AppState>,
    request: DropColumnsRequestV1,
) -> Result<ResultEnvelope<DropColumnsResponseV1>, String> {
    Ok(services_v1::drop_columns_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn write_rows_v1(
    state: tauri::State<'_, AppState>,
    request: WriteRowsRequestV1,
) -> Result<ResultEnvelope<WriteRowsResponseV1>, String> {
    Ok(services_v1::write_rows_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn update_rows_v1(
    state: tauri::State<'_, AppState>,
    request: UpdateRowsRequestV1,
) -> Result<ResultEnvelope<UpdateRowsResponseV1>, String> {
    Ok(services_v1::update_rows_v1(state.inner(), request).await)
}

#[tauri::command]
pub async fn delete_rows_v1(
    state: tauri::State<'_, AppState>,
    request: DeleteRowsRequestV1,
) -> Result<ResultEnvelope<DeleteRowsResponseV1>, String> {
    Ok(services_v1::delete_rows_v1(state.inner(), request).await)
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
