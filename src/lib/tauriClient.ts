import { invoke } from "@tauri-apps/api/core";

import type {
	ConnectProfile,
	ConnectResponseV1,
	ListTablesResponseV1,
	ResultEnvelope,
	ScanRequestV1,
	ScanResponseV1,
	SchemaDefinition,
	TableHandle,
} from "../ipc/v1";

async function invokeV1<T>(command: string, payload: unknown): Promise<ResultEnvelope<T>> {
	return invoke<ResultEnvelope<T>>(command, payload as never);
}

export function unwrapEnvelope<T>(envelope: ResultEnvelope<T>): T {
	if (!envelope.ok || envelope.data === undefined) {
		const message = envelope.error?.message ?? "unknown error";
		throw new Error(message);
	}
	return envelope.data;
}

export async function connectV1(profile: ConnectProfile): Promise<ResultEnvelope<ConnectResponseV1>> {
	return invokeV1("connect_v1", { profile });
}

export async function listTablesV1(connectionId: string): Promise<ResultEnvelope<ListTablesResponseV1>> {
	return invokeV1("list_tables_v1", { connectionId });
}

export async function openTableV1(
	connectionId: string,
	tableName: string,
): Promise<ResultEnvelope<TableHandle>> {
	return invokeV1("open_table_v1", { connectionId, tableName });
}

export async function getSchemaV1(tableId: string): Promise<ResultEnvelope<SchemaDefinition>> {
	return invokeV1("get_schema_v1", { tableId });
}

export async function scanV1(request: ScanRequestV1): Promise<ResultEnvelope<ScanResponseV1>> {
	return invokeV1("scan_v1", request);
}
