import { invoke } from "@tauri-apps/api/core"

import type {
	AddColumnsResponseV1,
	AlterColumnsResponseV1,
	CheckoutTableLatestRequestV1,
	CheckoutTableLatestResponseV1,
	CheckoutTableVersionRequestV1,
	CheckoutTableVersionResponseV1,
	CloneTableRequestV1,
	CloneTableResponseV1,
	CombinedSearchRequestV1,
	ConnectProfile,
	ConnectResponseV1,
	CreateTableResponseV1,
	CreateIndexResponseV1,
	DisconnectResponseV1,
	DropColumnsResponseV1,
	DropIndexResponseV1,
	DropTableResponseV1,
	FieldDataType,
	FtsSearchRequestV1,
	GetTableVersionRequestV1,
	GetTableVersionResponseV1,
	IndexTypeV1,
	ListVersionsRequestV1,
	ListVersionsResponseV1,
	ListTablesResponseV1,
	ListIndexesResponseV1,
	DeleteRowsResponseV1,
	ExportDataRequestV1,
	ExportDataResponseV1,
	ImportDataRequestV1,
	ImportDataResponseV1,
	OptimizeTableRequestV1,
	OptimizeTableResponseV1,
	QueryFilterRequestV1,
	QueryResponseV1,
	RenameTableRequestV1,
	RenameTableResponseV1,
	ResultEnvelope,
	ScanRequestV1,
	ScanResponseV1,
	SchemaDefinition,
	SchemaDefinitionInput,
	TableHandle,
	UpdateRowsResponseV1,
	VectorSearchRequestV1,
	WriteDataMode,
	WriteRowsResponseV1,
} from "../ipc/v1"

function normalizeInvokeError(error: unknown, fallback: string): Error {
	if (error instanceof Error) {
		return error
	}
	if (typeof error === "string") {
		return new Error(error)
	}
	if (typeof error === "object" && error && "message" in error) {
		const message = (error as { message?: unknown }).message
		if (typeof message === "string" && message.trim()) {
			return new Error(message)
		}
	}
	try {
		return new Error(JSON.stringify(error))
	} catch {
		return new Error(fallback)
	}
}

async function invokeV1<T>(command: string, payload: unknown): Promise<ResultEnvelope<T>> {
	try {
		return await invoke<ResultEnvelope<T>>(command, payload as never)
	} catch (error) {
		throw normalizeInvokeError(error, `调用 ${command} 失败`)
	}
}

export function unwrapEnvelope<T>(envelope: ResultEnvelope<T>): T {
	if (!envelope.ok || envelope.data === undefined) {
		const message = envelope.error?.message ?? "unknown error"
		throw new Error(message)
	}
	return envelope.data
}

export async function connectV1(
	profile: ConnectProfile
): Promise<ResultEnvelope<ConnectResponseV1>> {
	return invokeV1("connect_v1", { request: { profile } })
}

export async function disconnectV1(
	connectionId: string
): Promise<ResultEnvelope<DisconnectResponseV1>> {
	return invokeV1("disconnect_v1", { request: { connectionId } })
}

export async function listTablesV1(
	connectionId: string
): Promise<ResultEnvelope<ListTablesResponseV1>> {
	return invokeV1("list_tables_v1", { request: { connectionId } })
}

export async function dropTableV1(
	connectionId: string,
	tableName: string,
	namespace?: string[]
): Promise<ResultEnvelope<DropTableResponseV1>> {
	return invokeV1("drop_table_v1", { request: { connectionId, tableName, namespace } })
}

export async function renameTableV1(
	request: RenameTableRequestV1
): Promise<ResultEnvelope<RenameTableResponseV1>> {
	return invokeV1("rename_table_v1", { request })
}

export async function listIndexesV1(
	tableId: string
): Promise<ResultEnvelope<ListIndexesResponseV1>> {
	return invokeV1("list_indexes_v1", { request: { tableId } })
}

export async function createIndexV1(request: {
	tableId: string
	columns: string[]
	indexType: IndexTypeV1
	name?: string
	replace?: boolean
}): Promise<ResultEnvelope<CreateIndexResponseV1>> {
	return invokeV1("create_index_v1", { request })
}

export async function dropIndexV1(
	tableId: string,
	indexName: string
): Promise<ResultEnvelope<DropIndexResponseV1>> {
	return invokeV1("drop_index_v1", { request: { tableId, indexName } })
}

export async function createTableV1(
	connectionId: string,
	tableName: string,
	schema: SchemaDefinitionInput
): Promise<ResultEnvelope<CreateTableResponseV1>> {
	return invokeV1("create_table_v1", { request: { connectionId, tableName, schema } })
}

export async function openTableV1(
	connectionId: string,
	tableName: string
): Promise<ResultEnvelope<TableHandle>> {
	return invokeV1("open_table_v1", { request: { connectionId, tableName } })
}

export async function getSchemaV1(tableId: string): Promise<ResultEnvelope<SchemaDefinition>> {
	return invokeV1("get_schema_v1", { request: { tableId } })
}

export async function listVersionsV1(
	request: ListVersionsRequestV1
): Promise<ResultEnvelope<ListVersionsResponseV1>> {
	return invokeV1("list_versions_v1", { request })
}

export async function getTableVersionV1(
	request: GetTableVersionRequestV1
): Promise<ResultEnvelope<GetTableVersionResponseV1>> {
	return invokeV1("get_table_version_v1", { request })
}

export async function checkoutTableVersionV1(
	request: CheckoutTableVersionRequestV1
): Promise<ResultEnvelope<CheckoutTableVersionResponseV1>> {
	return invokeV1("checkout_table_version_v1", { request })
}

export async function checkoutTableLatestV1(
	request: CheckoutTableLatestRequestV1
): Promise<ResultEnvelope<CheckoutTableLatestResponseV1>> {
	return invokeV1("checkout_table_latest_v1", { request })
}

export async function cloneTableV1(
	request: CloneTableRequestV1
): Promise<ResultEnvelope<CloneTableResponseV1>> {
	return invokeV1("clone_table_v1", { request })
}

export async function addColumnsV1(
	tableId: string,
	columns: SchemaDefinitionInput
): Promise<ResultEnvelope<AddColumnsResponseV1>> {
	return invokeV1("add_columns_v1", { request: { tableId, columns } })
}

export async function alterColumnsV1(
	request: {
		tableId: string
		columns: {
			path: string
			rename?: string
			nullable?: boolean
			dataType?: FieldDataType
			vectorLength?: number
		}[]
	}
): Promise<ResultEnvelope<AlterColumnsResponseV1>> {
	return invokeV1("alter_columns_v1", { request })
}

export async function dropColumnsV1(
	tableId: string,
	columns: string[]
): Promise<ResultEnvelope<DropColumnsResponseV1>> {
	return invokeV1("drop_columns_v1", { request: { tableId, columns } })
}

export async function scanV1(request: ScanRequestV1): Promise<ResultEnvelope<ScanResponseV1>> {
	return invokeV1("scan_v1", { request })
}

export async function writeRowsV1(
	tableId: string,
	rows: unknown[],
	mode: WriteDataMode
): Promise<ResultEnvelope<WriteRowsResponseV1>> {
	return invokeV1("write_rows_v1", { request: { tableId, rows, mode } })
}

export async function updateRowsV1(
	request: {
		tableId: string
		filter?: string
		updates: { column: string; expr: string }[]
	}
): Promise<ResultEnvelope<UpdateRowsResponseV1>> {
	return invokeV1("update_rows_v1", { request })
}

export async function deleteRowsV1(
	tableId: string,
	filter: string
): Promise<ResultEnvelope<DeleteRowsResponseV1>> {
	return invokeV1("delete_rows_v1", { request: { tableId, filter } })
}

export async function importDataV1(
	request: ImportDataRequestV1
): Promise<ResultEnvelope<ImportDataResponseV1>> {
	return invokeV1("import_data_v1", { request })
}

export async function exportDataV1(
	request: ExportDataRequestV1
): Promise<ResultEnvelope<ExportDataResponseV1>> {
	return invokeV1("export_data_v1", { request })
}

export async function optimizeTableV1(
	request: OptimizeTableRequestV1
): Promise<ResultEnvelope<OptimizeTableResponseV1>> {
	return invokeV1("optimize_table_v1", { request })
}

export async function queryFilterV1(
	request: QueryFilterRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("query_filter_v1", { request })
}

export async function combinedSearchV1(
	request: CombinedSearchRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("combined_search_v1", { request })
}

export async function vectorSearchV1(
	request: VectorSearchRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("vector_search_v1", { request })
}

export async function ftsSearchV1(
	request: FtsSearchRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("fts_search_v1", { request })
}
