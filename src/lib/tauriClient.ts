import { invoke } from "@tauri-apps/api/core"

import type {
	ConnectProfile,
	ConnectResponseV1,
	FtsSearchRequestV1,
	ListTablesResponseV1,
	QueryFilterRequestV1,
	QueryResponseV1,
	ResultEnvelope,
	ScanRequestV1,
	ScanResponseV1,
	SchemaDefinition,
	TableHandle,
	VectorSearchRequestV1,
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
	return invokeV1("connect_v1", { profile })
}

export async function listTablesV1(
	connectionId: string
): Promise<ResultEnvelope<ListTablesResponseV1>> {
	return invokeV1("list_tables_v1", { connectionId })
}

export async function openTableV1(
	connectionId: string,
	tableName: string
): Promise<ResultEnvelope<TableHandle>> {
	return invokeV1("open_table_v1", { connectionId, tableName })
}

export async function getSchemaV1(tableId: string): Promise<ResultEnvelope<SchemaDefinition>> {
	return invokeV1("get_schema_v1", { tableId })
}

export async function scanV1(request: ScanRequestV1): Promise<ResultEnvelope<ScanResponseV1>> {
	return invokeV1("scan_v1", request)
}

export async function queryFilterV1(
	request: QueryFilterRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("query_filter_v1", request)
}

export async function vectorSearchV1(
	request: VectorSearchRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("vector_search_v1", request)
}

export async function ftsSearchV1(
	request: FtsSearchRequestV1
): Promise<ResultEnvelope<QueryResponseV1>> {
	return invokeV1("fts_search_v1", request)
}
