export type ApiVersion = "v1"

export type ErrorCode = "invalid_argument" | "not_found" | "internal" | "not_implemented"

export interface ErrorEnvelope {
	code: ErrorCode
	message: string
	details?: unknown
}

export interface ResultEnvelope<T> {
	apiVersion: ApiVersion
	ok: boolean
	data?: T
	error?: ErrorEnvelope
}

export type DataFormat = "json" | "arrow"

export type DataFileFormatV1 = "csv" | "parquet" | "jsonl"

export type WriteDataMode = "append" | "overwrite"

export type IndexTypeV1 =
	| "auto"
	| "btree"
	| "bitmap"
	| "label_list"
	| "fts"
	| "ivf_flat"
	| "ivf_sq"
	| "ivf_pq"
	| "ivf_rq"
	| "ivf_hnsw_pq"
	| "ivf_hnsw_sq"

export type AuthDescriptor =
	| { type: "none" }
	| { type: "inline"; provider: string; params: Record<string, string> }
	| { type: "secret_ref"; provider: string; reference: string }

export interface ConnectOptions {
	readConsistencyIntervalSeconds?: number
}

export interface ConnectProfile {
	name: string
	uri: string
	storageOptions?: Record<string, string>
	options?: ConnectOptions
	auth?: AuthDescriptor
}

export type BackendKind = "local" | "s3" | "gcs" | "azure" | "remote" | "unknown"

export interface ConnectResponseV1 {
	connectionId: string
	backendKind: BackendKind
	name: string
	uri: string
}

export interface DisconnectRequestV1 {
	connectionId: string
}

export interface DisconnectResponseV1 {
	connectionId: string
	releasedTables: number
}

export interface TableInfo {
	name: string
}

export interface ListTablesResponseV1 {
	tables: TableInfo[]
}

export interface DropTableRequestV1 {
	connectionId: string
	tableName: string
	namespace?: string[]
}

export interface DropTableResponseV1 {
	tableName: string
}

export interface RenameTableRequestV1 {
	connectionId: string
	tableName: string
	newTableName: string
	namespace?: string[]
	newNamespace?: string[]
}

export interface RenameTableResponseV1 {
	tableName: string
	newTableName: string
}

export interface ListIndexesRequestV1 {
	tableId: string
}

export interface IndexDefinitionV1 {
	name: string
	indexType: IndexTypeV1
	columns: string[]
}

export interface ListIndexesResponseV1 {
	indexes: IndexDefinitionV1[]
}

export interface CreateIndexRequestV1 {
	tableId: string
	columns: string[]
	indexType: IndexTypeV1
	name?: string
	replace?: boolean
}

export interface CreateIndexResponseV1 {
	tableId: string
	indexType: IndexTypeV1
	columns: string[]
	name?: string
}

export interface DropIndexRequestV1 {
	tableId: string
	indexName: string
}

export interface DropIndexResponseV1 {
	tableId: string
	indexName: string
}

export interface TableHandle {
	tableId: string
	name: string
}

export interface SchemaField {
	name: string
	dataType: string
	nullable: boolean
	metadata?: Record<string, string>
}

export type FieldDataType =
	| "int8"
	| "int16"
	| "int32"
	| "int64"
	| "uint8"
	| "uint16"
	| "uint32"
	| "uint64"
	| "float32"
	| "float64"
	| "boolean"
	| "utf8"
	| "large_utf8"
	| "binary"
	| "large_binary"
	| "fixed_size_list_float32"

export interface SchemaFieldInput {
	name: string
	dataType: FieldDataType
	nullable: boolean
	metadata?: Record<string, string>
	vectorLength?: number
}

export interface SchemaDefinitionInput {
	fields: SchemaFieldInput[]
}

export interface SchemaDefinition {
	fields: SchemaField[]
}

export interface CreateTableRequestV1 {
	connectionId: string
	tableName: string
	schema: SchemaDefinitionInput
}

export interface CreateTableResponseV1 {
	tableId: string
	name: string
}

export interface AddColumnsRequestV1 {
	tableId: string
	columns: SchemaDefinitionInput
}

export interface AddColumnsResponseV1 {
	tableId: string
	added: string[]
	schema: SchemaDefinition
}

export interface ColumnAlterationInput {
	path: string
	rename?: string
	nullable?: boolean
	dataType?: FieldDataType
	vectorLength?: number
}

export interface AlterColumnsRequestV1 {
	tableId: string
	columns: ColumnAlterationInput[]
}

export interface AlterColumnsResponseV1 {
	tableId: string
	updated: string[]
	schema: SchemaDefinition
}

export interface DropColumnsRequestV1 {
	tableId: string
	columns: string[]
}

export interface DropColumnsResponseV1 {
	tableId: string
	dropped: string[]
	schema: SchemaDefinition
}

export interface ScanRequestV1 {
	tableId: string
	format?: DataFormat
	projection?: string[]
	filter?: string
	limit?: number
	offset?: number
}

export interface WriteRowsRequestV1 {
	tableId: string
	rows: unknown[]
	mode?: WriteDataMode
}

export interface WriteRowsResponseV1 {
	tableId: string
	rows: number
	version: number
}

export interface UpdateColumnInputV1 {
	column: string
	expr: string
}

export interface UpdateRowsRequestV1 {
	tableId: string
	filter?: string
	updates: UpdateColumnInputV1[]
}

export interface UpdateRowsResponseV1 {
	tableId: string
	rowsUpdated: number
	version: number
}

export interface DeleteRowsRequestV1 {
	tableId: string
	filter: string
}

export interface DeleteRowsResponseV1 {
	tableId: string
	version: number
}

export interface ImportDataRequestV1 {
	tableId: string
	path: string
	format: DataFileFormatV1
	mode?: WriteDataMode
	hasHeader?: boolean
	delimiter?: string
}

export interface ImportDataResponseV1 {
	tableId: string
	rows: number
}

export interface ExportDataRequestV1 {
	tableId: string
	path: string
	format: DataFileFormatV1
	projection?: string[]
	filter?: string
	limit?: number
	offset?: number
	delimiter?: string
	withHeader?: boolean
}

export interface ExportDataResponseV1 {
	path: string
	rows: number
}

export type OptimizeActionV1 = "compact" | "vacuum"

export interface OptimizeTableRequestV1 {
	tableId: string
	action: OptimizeActionV1
	targetRowsPerFragment?: number
	olderThanDays?: number
	deleteUnverified?: boolean
	errorIfTaggedOldVersions?: boolean
}

export interface OptimizeTableResponseV1 {
	tableId: string
	action: OptimizeActionV1
	summary: string
}

export type DataChunk =
	| {
			format: "json"
			rows: unknown[]
			schema: SchemaDefinition
			offset: number
			limit: number
	  }
	| {
			format: "arrow"
			ipcBase64: string
			compression?: string
	  }

export interface ScanResponseV1 {
	chunk: DataChunk
	nextOffset?: number
}

export interface VersionInfoV1 {
	version: number
	timestamp: string
	metadata: Record<string, string>
}

export interface ListVersionsRequestV1 {
	tableId: string
}

export interface ListVersionsResponseV1 {
	versions: VersionInfoV1[]
}

export interface GetTableVersionRequestV1 {
	tableId: string
}

export interface GetTableVersionResponseV1 {
	tableId: string
	version: number
}

export interface CheckoutTableVersionRequestV1 {
	tableId: string
	version: number
}

export interface CheckoutTableVersionResponseV1 {
	tableId: string
	version: number
}

export interface CheckoutTableLatestRequestV1 {
	tableId: string
}

export interface CheckoutTableLatestResponseV1 {
	tableId: string
	version: number
}

export interface CloneTableRequestV1 {
	connectionId: string
	tableId: string
	targetTableName: string
	sourceVersion?: number
	sourceTag?: string
	isShallow?: boolean
}

export interface CloneTableResponseV1 {
	tableId: string
	name: string
}

export interface CombinedSearchRequestV1 {
	tableId: string
	vector?: number[]
	vectorColumn?: string
	query?: string
	columns?: string[]
	projection?: string[]
	filter?: string
	limit?: number
	offset?: number
	nprobes?: number
	refineFactor?: number
}

export interface VectorSearchRequestV1 {
	tableId: string
	vector: number[]
	column?: string
	topK?: number
	projection?: string[]
	filter?: string
	nprobes?: number
	refineFactor?: number
	offset?: number
}

export interface FtsSearchRequestV1 {
	tableId: string
	query: string
	columns?: string[]
	limit?: number
	offset?: number
	projection?: string[]
	filter?: string
}

export interface QueryFilterRequestV1 {
	tableId: string
	filter: string
	projection?: string[]
	limit?: number
	offset?: number
}

export interface QueryResponseV1 {
	chunk: DataChunk
	nextOffset?: number
}
