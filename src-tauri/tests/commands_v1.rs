use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, Int32Array, RecordBatch, RecordBatchIterator, StringArray};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use base64::{engine::general_purpose, Engine as _};
use lancedb::index::Index;
use tempfile::tempdir;

use lancedb_viewer_lib::ipc::v1::{
    AddColumnsRequestV1, AlterColumnsRequestV1, ColumnAlterationInput, ConnectProfile,
    ConnectRequestV1, CreateIndexRequestV1, CreateTableRequestV1, DataFormat,
    DeleteRowsRequestV1, DropColumnsRequestV1, DropIndexRequestV1, DropTableRequestV1, ErrorCode,
    FieldDataType, FtsSearchRequestV1, GetSchemaRequestV1, IndexTypeV1,
    ListIndexesRequestV1, ListTablesRequestV1, OpenTableRequestV1, QueryFilterRequestV1,
    ScanRequestV1, SchemaDefinitionInput, SchemaFieldInput, UpdateColumnInputV1,
    UpdateRowsRequestV1, VectorSearchRequestV1, WriteDataMode, WriteRowsRequestV1,
};
use lancedb_viewer_lib::services::v1 as services_v1;
use lancedb_viewer_lib::state::AppState;

struct SampleDb {
    _dir: tempfile::TempDir,
    uri: String,
    table_name: String,
}

fn sample_db_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("manifest dir parent")
        .join("sample-db")
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

async fn seed_items_table(uri: &str, table_name: &str, rows: usize) {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("text", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
    ]));

    let ids = Int32Array::from_iter_values(0..rows as i32);
    let texts = StringArray::from_iter_values((0..rows).map(|index| format!("item {index}")));
    let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        (0..rows).map(|index| {
            let base = index as f32 * 0.1;
            Some(vec![Some(base), Some(base + 0.1), Some(base + 0.2)])
        }),
        3,
    );

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(ids), Arc::new(texts), Arc::new(vectors)],
    )
    .expect("create record batch");

    let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);
    let db = lancedb::connect(uri)
        .execute()
        .await
        .expect("connect lancedb");

    db.create_table(table_name, Box::new(batches))
        .execute()
        .await
        .expect("create table");
}

async fn prepare_sample_db() -> SampleDb {
    let table_name = "items".to_string();
    let sample_root = sample_db_root();

    if sample_root.exists() {
        let temp_dir = tempdir().expect("create tempdir");
        let dest = temp_dir.path().join("sample-db");
        copy_dir_all(&sample_root, &dest).expect("copy sample-db");
        return SampleDb {
            _dir: temp_dir,
            uri: dest.to_string_lossy().to_string(),
            table_name,
        };
    }

    let temp_dir = tempdir().expect("create tempdir");
    let uri = temp_dir.path().to_string_lossy().to_string();
    seed_items_table(&uri, &table_name, 50).await;

    SampleDb {
        _dir: temp_dir,
        uri,
        table_name,
    }
}

struct CommandHarness {
    _db: SampleDb,
    state: AppState,
    connection_id: String,
    table_id: String,
    table_name: String,
}

async fn create_command_harness() -> CommandHarness {
    let sample = prepare_sample_db().await;
    let state = AppState::new();

    let connect = services_v1::connect_v1(
        &state,
        ConnectRequestV1 {
            profile: ConnectProfile {
                name: "sample".to_string(),
                uri: sample.uri.clone(),
                storage_options: Default::default(),
                options: Default::default(),
                auth: Default::default(),
            },
        },
    )
    .await;

    assert!(connect.ok, "connect should succeed: {:?}", connect.error);
    let connection_id = connect
        .data
        .as_ref()
        .expect("connect data")
        .connection_id
        .clone();

    let opened = services_v1::open_table_v1(
        &state,
        OpenTableRequestV1 {
            connection_id: connection_id.clone(),
            table_name: sample.table_name.clone(),
        },
    )
    .await;

    assert!(opened.ok, "open_table should succeed: {:?}", opened.error);
    let table_id = opened.data.expect("table handle").table_id;

    CommandHarness {
        _db: sample,
        state,
        connection_id,
        table_id,
        table_name: "items".to_string(),
    }
}

#[tokio::test]
async fn list_tables_and_get_schema() {
    let harness = create_command_harness().await;

    let listed = services_v1::list_tables_v1(
        &harness.state,
        ListTablesRequestV1 {
            connection_id: harness.connection_id.clone(),
        },
    )
    .await;

    assert!(listed.ok, "list_tables should succeed: {:?}", listed.error);
    let tables = listed.data.expect("tables").tables;
    assert!(
        tables.iter().any(|table| table.name == harness.table_name),
        "expected sample table to exist"
    );

    let schema = services_v1::get_schema_v1(
        &harness.state,
        GetSchemaRequestV1 {
            table_id: harness.table_id.clone(),
        },
    )
    .await;

    assert!(schema.ok, "get_schema should succeed: {:?}", schema.error);
    assert!(
        schema
            .data
            .expect("schema")
            .fields
            .iter()
            .any(|field| field.name == "id"),
        "schema should include id field"
    );
}

#[tokio::test]
async fn drop_table_removes_table() {
    let harness = create_command_harness().await;

    let dropped = services_v1::drop_table_v1(
        &harness.state,
        DropTableRequestV1 {
            connection_id: harness.connection_id.clone(),
            table_name: harness.table_name.clone(),
            namespace: None,
        },
    )
    .await;

    assert!(dropped.ok, "drop_table should succeed: {:?}", dropped.error);

    let listed = services_v1::list_tables_v1(
        &harness.state,
        ListTablesRequestV1 {
            connection_id: harness.connection_id.clone(),
        },
    )
    .await;

    assert!(listed.ok, "list_tables should succeed: {:?}", listed.error);
    let tables = listed.data.expect("tables").tables;
    assert!(
        !tables.iter().any(|table| table.name == harness.table_name),
        "expected dropped table to be removed"
    );
}

#[tokio::test]
async fn create_table_and_schema_evolution() {
    let harness = create_command_harness().await;

    let created = services_v1::create_table_v1(
        &harness.state,
        CreateTableRequestV1 {
            connection_id: harness.connection_id.clone(),
            table_name: "created_table".to_string(),
            schema: SchemaDefinitionInput {
                fields: vec![
                    SchemaFieldInput {
                        name: "id".to_string(),
                        data_type: FieldDataType::Int32,
                        nullable: false,
                        metadata: None,
                        vector_length: None,
                    },
                    SchemaFieldInput {
                        name: "name".to_string(),
                        data_type: FieldDataType::Utf8,
                        nullable: true,
                        metadata: None,
                        vector_length: None,
                    },
                ],
            },
        },
    )
    .await;

    assert!(created.ok, "create_table should succeed: {:?}", created.error);
    let created = created.data.expect("create table data");

    let added = services_v1::add_columns_v1(
        &harness.state,
        AddColumnsRequestV1 {
            table_id: created.table_id.clone(),
            columns: SchemaDefinitionInput {
                fields: vec![SchemaFieldInput {
                    name: "notes".to_string(),
                    data_type: FieldDataType::Utf8,
                    nullable: true,
                    metadata: None,
                    vector_length: None,
                }],
            },
        },
    )
    .await;

    assert!(added.ok, "add_columns should succeed: {:?}", added.error);
    let added = added.data.expect("add_columns data");
    assert!(
        added.schema.fields.iter().any(|field| field.name == "notes"),
        "expected notes column to be added"
    );

    let altered = services_v1::alter_columns_v1(
        &harness.state,
        AlterColumnsRequestV1 {
            table_id: created.table_id.clone(),
            columns: vec![ColumnAlterationInput {
                path: "notes".to_string(),
                rename: Some("notes_text".to_string()),
                nullable: None,
                data_type: None,
                vector_length: None,
            }],
        },
    )
    .await;

    assert!(altered.ok, "alter_columns should succeed: {:?}", altered.error);
    let altered = altered.data.expect("alter_columns data");
    assert!(
        altered
            .schema
            .fields
            .iter()
            .any(|field| field.name == "notes_text"),
        "expected notes column to be renamed"
    );
    assert!(
        !altered
            .schema
            .fields
            .iter()
            .any(|field| field.name == "notes"),
        "expected old notes column to be removed"
    );

    let dropped = services_v1::drop_columns_v1(
        &harness.state,
        DropColumnsRequestV1 {
            table_id: created.table_id.clone(),
            columns: vec!["notes_text".to_string()],
        },
    )
    .await;

    assert!(dropped.ok, "drop_columns should succeed: {:?}", dropped.error);
    let dropped = dropped.data.expect("drop_columns data");
    assert!(
        !dropped
            .schema
            .fields
            .iter()
            .any(|field| field.name == "notes_text"),
        "expected notes_text column to be dropped"
    );

    let cleanup = services_v1::drop_table_v1(
        &harness.state,
        DropTableRequestV1 {
            connection_id: harness.connection_id.clone(),
            table_name: created.name,
            namespace: None,
        },
    )
    .await;

    assert!(cleanup.ok, "cleanup drop_table should succeed: {:?}", cleanup.error);
}

#[tokio::test]
async fn write_update_delete_rows() {
    let harness = create_command_harness().await;

    let write = services_v1::write_rows_v1(
        &harness.state,
        WriteRowsRequestV1 {
            table_id: harness.table_id.clone(),
            rows: vec![
                serde_json::json!({"id": 999, "text": "new", "vector": [0.1, 0.2, 0.3]}),
                serde_json::json!({"id": 1000, "text": "new", "vector": [0.2, 0.3, 0.4]}),
            ],
            mode: WriteDataMode::Append,
        },
    )
    .await;

    assert!(write.ok, "write_rows should succeed: {:?}", write.error);

    let updated = services_v1::update_rows_v1(
        &harness.state,
        UpdateRowsRequestV1 {
            table_id: harness.table_id.clone(),
            filter: Some("id = 999".to_string()),
            updates: vec![UpdateColumnInputV1 {
                column: "text".to_string(),
                expr: "'updated'".to_string(),
            }],
        },
    )
    .await;

    assert!(updated.ok, "update_rows should succeed: {:?}", updated.error);
    let updated = updated.data.expect("update rows data");
    assert!(updated.rows_updated >= 1);

    let deleted = services_v1::delete_rows_v1(
        &harness.state,
        DeleteRowsRequestV1 {
            table_id: harness.table_id.clone(),
            filter: "id = 999".to_string(),
        },
    )
    .await;

    assert!(deleted.ok, "delete_rows should succeed: {:?}", deleted.error);
}

#[tokio::test]
async fn scan_json_and_arrow() {
    let harness = create_command_harness().await;

    let scan_page1 = services_v1::scan_v1(
        &harness.state,
        ScanRequestV1 {
            table_id: harness.table_id.clone(),
            format: DataFormat::Json,
            projection: None,
            filter: None,
            limit: Some(2),
            offset: Some(0),
        },
    )
    .await;

    assert!(scan_page1.ok, "scan json should succeed: {:?}", scan_page1.error);
    let scan_page1 = scan_page1.data.expect("scan data");
    assert_eq!(scan_page1.next_offset, Some(2));
    match scan_page1.chunk {
        lancedb_viewer_lib::ipc::v1::DataChunk::Json(chunk) => assert_eq!(chunk.rows.len(), 2),
        _ => panic!("expected json chunk"),
    }

    let scan_arrow = services_v1::scan_v1(
        &harness.state,
        ScanRequestV1 {
            table_id: harness.table_id.clone(),
            format: DataFormat::Arrow,
            projection: None,
            filter: None,
            limit: Some(3),
            offset: Some(0),
        },
    )
    .await;

    assert!(scan_arrow.ok, "scan arrow should succeed: {:?}", scan_arrow.error);
    let scan_arrow = scan_arrow.data.expect("scan arrow");
    assert_eq!(scan_arrow.next_offset, Some(3));

    let ipc_base64 = match scan_arrow.chunk {
        lancedb_viewer_lib::ipc::v1::DataChunk::Arrow(chunk) => chunk.ipc_base64,
        _ => panic!("expected arrow chunk"),
    };

    let decoded = general_purpose::STANDARD
        .decode(ipc_base64)
        .expect("decode base64");
    let reader = StreamReader::try_new(Cursor::new(decoded), None).expect("open stream reader");
    let row_count: usize = reader
        .map(|batch| batch.expect("read batch").num_rows())
        .sum();

    assert_eq!(row_count, 3);
}

#[tokio::test]
async fn query_filter_vector_search_and_fts() {
    let harness = create_command_harness().await;

    let filtered = services_v1::query_filter_v1(
        &harness.state,
        QueryFilterRequestV1 {
            table_id: harness.table_id.clone(),
            filter: "id >= 2".to_string(),
            projection: None,
            limit: Some(2),
            offset: Some(0),
        },
    )
    .await;

    assert!(filtered.ok, "query_filter should succeed: {:?}", filtered.error);
    let filtered = filtered.data.expect("filtered data");
    match filtered.chunk {
        lancedb_viewer_lib::ipc::v1::DataChunk::Json(chunk) => {
            assert!(chunk.rows.len() <= 2)
        }
        _ => panic!("expected json chunk"),
    }

    let vector_ok = services_v1::vector_search_v1(
        &harness.state,
        VectorSearchRequestV1 {
            table_id: harness.table_id.clone(),
            vector: vec![0.0, 0.1, 0.2],
            column: Some("vector".to_string()),
            top_k: Some(2),
            projection: None,
            filter: None,
            nprobes: None,
            refine_factor: None,
            offset: Some(0),
        },
    )
    .await;

    assert!(vector_ok.ok, "vector_search should succeed: {:?}", vector_ok.error);
    let vector_ok = vector_ok.data.expect("vector data");
    match vector_ok.chunk {
        lancedb_viewer_lib::ipc::v1::DataChunk::Json(chunk) => {
            assert!(!chunk.rows.is_empty())
        }
        _ => panic!("expected json chunk"),
    }

    let table = harness
        .state
        .connections
        .lock()
        .expect("lock")
        .get_table(&harness.table_id)
        .expect("table");

    table
        .create_index(&["text"], Index::FTS(Default::default()))
        .execute()
        .await
        .expect("create fts index");

    let fts_ok = services_v1::fts_search_v1(
        &harness.state,
        FtsSearchRequestV1 {
            table_id: harness.table_id.clone(),
            query: "item 1".to_string(),
            columns: Some(vec!["text".to_string()]),
            limit: Some(5),
            offset: Some(0),
            projection: None,
            filter: None,
        },
    )
    .await;

    assert!(fts_ok.ok, "fts_search should succeed: {:?}", fts_ok.error);
    let fts_ok = fts_ok.data.expect("fts data");
    match fts_ok.chunk {
        lancedb_viewer_lib::ipc::v1::DataChunk::Json(chunk) => {
            assert!(!chunk.rows.is_empty())
        }
        _ => panic!("expected json chunk"),
    }
}

#[tokio::test]
async fn list_create_drop_indexes() {
    let harness = create_command_harness().await;

    let listed = services_v1::list_indexes_v1(
        &harness.state,
        ListIndexesRequestV1 {
            table_id: harness.table_id.clone(),
        },
    )
    .await;

    assert!(listed.ok, "list_indexes should succeed: {:?}", listed.error);

    let created = services_v1::create_index_v1(
        &harness.state,
        CreateIndexRequestV1 {
            table_id: harness.table_id.clone(),
            columns: vec!["id".to_string()],
            index_type: IndexTypeV1::BTree,
            name: Some("id_btree".to_string()),
            replace: true,
        },
    )
    .await;

    assert!(created.ok, "create_index should succeed: {:?}", created.error);

    let listed_after = services_v1::list_indexes_v1(
        &harness.state,
        ListIndexesRequestV1 {
            table_id: harness.table_id.clone(),
        },
    )
    .await;

    assert!(
        listed_after.ok,
        "list_indexes after create should succeed: {:?}",
        listed_after.error
    );
    let indexes = listed_after.data.expect("index list").indexes;
    assert!(
        indexes.iter().any(|index| index.name == "id_btree"),
        "expected id_btree index to exist"
    );

    let dropped = services_v1::drop_index_v1(
        &harness.state,
        DropIndexRequestV1 {
            table_id: harness.table_id.clone(),
            index_name: "id_btree".to_string(),
        },
    )
    .await;

    assert!(dropped.ok, "drop_index should succeed: {:?}", dropped.error);

    let listed_final = services_v1::list_indexes_v1(
        &harness.state,
        ListIndexesRequestV1 {
            table_id: harness.table_id.clone(),
        },
    )
    .await;

    assert!(
        listed_final.ok,
        "list_indexes after drop should succeed: {:?}",
        listed_final.error
    );
    let indexes = listed_final.data.expect("index list").indexes;
    assert!(
        !indexes.iter().any(|index| index.name == "id_btree"),
        "expected id_btree index to be removed"
    );
}

#[tokio::test]
async fn validates_error_conditions() {
    let harness = create_command_harness().await;

    let invalid_filter = services_v1::query_filter_v1(
        &harness.state,
        QueryFilterRequestV1 {
            table_id: harness.table_id.clone(),
            filter: " ".to_string(),
            projection: None,
            limit: None,
            offset: None,
        },
    )
    .await;

    assert!(!invalid_filter.ok);
    assert_eq!(
        invalid_filter.error.expect("error").code,
        ErrorCode::InvalidArgument
    );

    let empty_vector = services_v1::vector_search_v1(
        &harness.state,
        VectorSearchRequestV1 {
            table_id: harness.table_id.clone(),
            vector: vec![],
            column: Some("vector".to_string()),
            top_k: None,
            projection: None,
            filter: None,
            nprobes: None,
            refine_factor: None,
            offset: None,
        },
    )
    .await;

    assert!(!empty_vector.ok);
    assert_eq!(
        empty_vector.error.expect("error").code,
        ErrorCode::InvalidArgument
    );

    let missing_schema = services_v1::get_schema_v1(
        &harness.state,
        GetSchemaRequestV1 {
            table_id: "missing".to_string(),
        },
    )
    .await;

    assert!(!missing_schema.ok);
    assert_eq!(missing_schema.error.expect("error").code, ErrorCode::NotFound);
}
