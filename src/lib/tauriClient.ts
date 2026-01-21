import { invoke } from "@tauri-apps/api/core"

import type {
	AddColumnsResponseV1,
	AlterColumnsResponseV1,
	ConnectProfile,
	ConnectResponseV1,
	CreateTableResponseV1,
	CreateIndexResponseV1,
	DropColumnsResponseV1,
	DropIndexResponseV1,
	DropTableResponseV1,
	FieldDataType,
	FtsSearchRequestV1,
	IndexTypeV1,
	ListTablesResponseV1,
	ListIndexesResponseV1,
	DeleteRowsResponseV1,
	QueryFilterRequestV1,
	QueryResponseV1,
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

export async function queryFilterV1(
	request: QueryFilterRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("query_filter_v1", { request })
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
