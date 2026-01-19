export type ApiVersion = "v1";

export type ErrorCode = "invalid_argument" | "not_found" | "internal" | "not_implemented";

export interface ErrorEnvelope {
	code: ErrorCode;
	message: string;
	details?: unknown;
}

export interface ResultEnvelope<T> {
	apiVersion: ApiVersion;
	ok: boolean;
	data?: T;
	error?: ErrorEnvelope;
}

export type DataFormat = "json" | "arrow";

export type AuthDescriptor =
	| { type: "none" }
	| { type: "inline"; provider: string; params: Record<string, string> }
	| { type: "secret_ref"; provider: string; reference: string };

export interface ConnectOptions {
	readConsistencyIntervalSeconds?: number;
}

export interface ConnectProfile {
	name: string;
	uri: string;
	storageOptions?: Record<string, string>;
	options?: ConnectOptions;
	auth?: AuthDescriptor;
}

export type BackendKind = "local" | "s3" | "gcs" | "azure" | "remote" | "unknown";

export interface ConnectResponseV1 {
	connectionId: string;
	backendKind: BackendKind;
	name: string;
	uri: string;
}

export interface TableInfo {
	name: string;
}

export interface ListTablesResponseV1 {
	tables: TableInfo[];
}

export interface TableHandle {
	tableId: string;
	name: string;
}

export interface SchemaField {
	name: string;
	dataType: string;
	nullable: boolean;
	metadata?: Record<string, string>;
}

export interface SchemaDefinition {
	fields: SchemaField[];
}

export interface ScanRequestV1 {
	tableId: string;
	format?: DataFormat;
	projection?: string[];
	filter?: string;
	limit?: number;
	offset?: number;
}

export type DataChunk =
	| {
			format: "json";
			rows: unknown[];
			schema: SchemaDefinition;
			offset: number;
			limit: number;
	  }
	| {
			format: "arrow";
			ipcBase64: string;
			compression?: string;
	  };

export interface ScanResponseV1 {
	chunk: DataChunk;
	nextOffset?: number;
}
