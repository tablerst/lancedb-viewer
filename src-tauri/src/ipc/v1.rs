use arrow_schema::Schema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::domain::connect::BackendKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiVersion {
    V1,
}

impl Default for ApiVersion {
    fn default() -> Self {
        ApiVersion::V1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidArgument,
    NotFound,
    Internal,
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultEnvelope<T> {
    #[serde(default)]
    pub api_version: ApiVersion,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorEnvelope>,
}

impl<T> ResultEnvelope<T> {
    pub fn ok(data: T) -> Self {
        Self {
            api_version: ApiVersion::V1,
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            api_version: ApiVersion::V1,
            ok: false,
            data: None,
            error: Some(ErrorEnvelope {
                code,
                message: message.into(),
                details: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataFormat {
    Json,
    Arrow,
}

impl Default for DataFormat {
    fn default() -> Self {
        DataFormat::Json
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteDataMode {
    Append,
    Overwrite,
}

impl Default for WriteDataMode {
    fn default() -> Self {
        WriteDataMode::Append
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexTypeV1 {
    Auto,
    BTree,
    Bitmap,
    LabelList,
    Fts,
    IvfFlat,
    IvfSq,
    IvfPq,
    IvfRq,
    IvfHnswPq,
    IvfHnswSq,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthDescriptor {
    None,
    Inline {
        provider: String,
        params: HashMap<String, String>,
    },
    SecretRef {
        provider: String,
        reference: String,
    },
}

impl Default for AuthDescriptor {
    fn default() -> Self {
        AuthDescriptor::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_consistency_interval_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectProfile {
    pub name: String,
    pub uri: String,
    #[serde(default)]
    pub storage_options: HashMap<String, String>,
    #[serde(default)]
    pub options: ConnectOptions,
    #[serde(default)]
    pub auth: AuthDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequestV1 {
    pub profile: ConnectProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponseV1 {
    pub connection_id: String,
    pub backend_kind: BackendKind,
    pub name: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTablesRequestV1 {
    pub connection_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTablesResponseV1 {
    pub tables: Vec<TableInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropTableRequestV1 {
    pub connection_id: String,
    pub table_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropTableResponseV1 {
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIndexesRequestV1 {
    pub table_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexDefinitionV1 {
    pub name: String,
    pub index_type: IndexTypeV1,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIndexesResponseV1 {
    pub indexes: Vec<IndexDefinitionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexRequestV1 {
    pub table_id: String,
    pub columns: Vec<String>,
    pub index_type: IndexTypeV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub replace: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexResponseV1 {
    pub table_id: String,
    pub index_type: IndexTypeV1,
    pub columns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropIndexRequestV1 {
    pub table_id: String,
    pub index_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropIndexResponseV1 {
    pub table_id: String,
    pub index_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTableRequestV1 {
    pub connection_id: String,
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableHandle {
    pub table_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaRequestV1 {
    pub table_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldDataType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Boolean,
    Utf8,
    LargeUtf8,
    Binary,
    LargeBinary,
    FixedSizeListFloat32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaFieldInput {
    pub name: String,
    pub data_type: FieldDataType,
    pub nullable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_length: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefinitionInput {
    pub fields: Vec<SchemaFieldInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaField {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDefinition {
    pub fields: Vec<SchemaField>,
}

impl SchemaDefinition {
    pub fn from_arrow_schema(schema: &Schema) -> Self {
        let fields = schema
            .fields()
            .iter()
            .map(|field| SchemaField {
                name: field.name().to_string(),
                data_type: format!("{:?}", field.data_type()),
                nullable: field.is_nullable(),
                metadata: if field.metadata().is_empty() {
                    None
                } else {
                    Some(field.metadata().clone())
                },
            })
            .collect();

        Self { fields }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequestV1 {
    pub table_id: String,
    #[serde(default)]
    pub format: DataFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteRowsRequestV1 {
    pub table_id: String,
    pub rows: Vec<serde_json::Value>,
    #[serde(default)]
    pub mode: WriteDataMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteRowsResponseV1 {
    pub table_id: String,
    pub rows: usize,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateColumnInputV1 {
    pub column: String,
    pub expr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRowsRequestV1 {
    pub table_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    pub updates: Vec<UpdateColumnInputV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRowsResponseV1 {
    pub table_id: String,
    pub rows_updated: u64,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRowsRequestV1 {
    pub table_id: String,
    pub filter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteRowsResponseV1 {
    pub table_id: String,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTableRequestV1 {
    pub connection_id: String,
    pub table_name: String,
    pub schema: SchemaDefinitionInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTableResponseV1 {
    pub table_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddColumnsRequestV1 {
    pub table_id: String,
    pub columns: SchemaDefinitionInput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddColumnsResponseV1 {
    pub table_id: String,
    pub added: Vec<String>,
    pub schema: SchemaDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnAlterationInput {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<FieldDataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_length: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterColumnsRequestV1 {
    pub table_id: String,
    pub columns: Vec<ColumnAlterationInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlterColumnsResponseV1 {
    pub table_id: String,
    pub updated: Vec<String>,
    pub schema: SchemaDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropColumnsRequestV1 {
    pub table_id: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropColumnsResponseV1 {
    pub table_id: String,
    pub dropped: Vec<String>,
    pub schema: SchemaDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonChunk {
    pub rows: Vec<serde_json::Value>,
    pub schema: SchemaDefinition,
    pub offset: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrowChunk {
    pub ipc_base64: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum DataChunk {
    Json(JsonChunk),
    Arrow(ArrowChunk),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponseV1 {
    pub chunk: DataChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorSearchRequestV1 {
    pub table_id: String,
    pub vector: Vec<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nprobes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refine_factor: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FtsSearchRequestV1 {
    pub table_id: String,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilterRequestV1 {
    pub table_id: String,
    pub filter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponseV1 {
    pub chunk: DataChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
}
