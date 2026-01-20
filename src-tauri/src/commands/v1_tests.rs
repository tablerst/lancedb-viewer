use std::sync::Arc;
use std::{io::Cursor, panic};

use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, Int32Array, RecordBatch, RecordBatchIterator, StringArray};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use base64::{engine::general_purpose, Engine as _};
use lancedb::index::Index;
use tempfile::tempdir;

use super::*;
use crate::ipc::v1::ConnectProfile;

struct TestTable {
    _dir: tempfile::TempDir,
    table: lancedb::Table,
}

async fn create_test_table() -> TestTable {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("text", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
    ]));

    let ids = Int32Array::from_iter_values(0..5);
    let texts = StringArray::from(vec!["alpha", "beta", "gamma", "delta", "epsilon"]);
    let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vec![
            Some(vec![Some(0.1), Some(0.2), Some(0.3)]),
            Some(vec![Some(0.0), Some(0.1), Some(0.0)]),
            Some(vec![Some(0.2), Some(0.1), Some(0.0)]),
            Some(vec![Some(0.9), Some(0.9), Some(0.9)]),
            Some(vec![Some(0.5), Some(0.4), Some(0.3)]),
        ],
        3,
    );

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(ids), Arc::new(texts), Arc::new(vectors)],
    )
    .expect("create record batch");

    let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);
    let dir = tempdir().expect("create tempdir");
    let db = lancedb::connect(dir.path().to_str().expect("tempdir path"))
        .execute()
        .await
        .expect("connect lancedb");
    let table = db
        .create_table("items", Box::new(batches))
        .execute()
        .await
        .expect("create table");

    TestTable { _dir: dir, table }
}

struct CommandHarness {
    _dir: tempfile::TempDir,
    state: AppState,
    connection_id: String,
    table_id: String,
}

async fn seed_items_table(connection: &lancedb::Connection) {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("text", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 3),
            false,
        ),
    ]));

    let ids = Int32Array::from_iter_values(0..5);
    let texts = StringArray::from(vec!["alpha", "beta", "gamma", "delta", "epsilon"]);
    let vectors = FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vec![
            Some(vec![Some(0.1), Some(0.2), Some(0.3)]),
            Some(vec![Some(0.0), Some(0.1), Some(0.0)]),
            Some(vec![Some(0.2), Some(0.1), Some(0.0)]),
            Some(vec![Some(0.9), Some(0.9), Some(0.9)]),
            Some(vec![Some(0.5), Some(0.4), Some(0.3)]),
        ],
        3,
    );

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(ids), Arc::new(texts), Arc::new(vectors)],
    )
    .expect("create record batch");

    let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);

    connection
        .create_table("items", Box::new(batches))
        .execute()
        .await
        .expect("create table");
}

async fn create_command_harness() -> CommandHarness {
    let dir = tempdir().expect("create tempdir");
    let uri = dir.path().to_str().expect("tempdir path").to_string();
    let state = AppState::new();

    let envelope = connect_v1_impl(
        &state,
        ConnectRequestV1 {
            profile: ConnectProfile {
                name: "test".to_string(),
                uri,
                storage_options: Default::default(),
                options: Default::default(),
                auth: Default::default(),
            },
        },
    )
    .await;

    assert!(envelope.ok, "connect should succeed: {:?}", envelope.error);
    let connection_id = envelope
        .data
        .as_ref()
        .expect("connect data")
        .connection_id
        .clone();

    let connection = state
        .connections
        .lock()
        .expect("lock connections")
        .get_connection(&connection_id)
        .expect("connection exists");

    seed_items_table(&connection).await;

    let opened = open_table_v1_impl(
        &state,
        OpenTableRequestV1 {
            connection_id: connection_id.clone(),
            table_name: "items".to_string(),
        },
    )
    .await;
    assert!(opened.ok, "open_table should succeed: {:?}", opened.error);

    let table_id = opened.data.expect("table handle").table_id;

    CommandHarness {
        _dir: dir,
        state,
        connection_id,
        table_id,
    }
}

fn poison_connections_mutex(state: &AppState) {
    let result = panic::catch_unwind(|| {
        let _guard = state.connections.lock().expect("lock for poison");
        panic!("poison mutex");
    });
    assert!(result.is_err(), "expected a panic to poison the mutex");
}

#[tokio::test]
async fn query_filter_returns_rows() {
    let test = create_test_table().await;
    let table = &test.table;
    let fallback_schema = SchemaDefinition::from_arrow_schema(
        table.schema().await.expect("schema").as_ref(),
    );
    let query = table.query().only_if("id >= 2").limit(2);
    let (rows, _) = execute_query_json(query, fallback_schema)
        .await
        .expect("execute query");
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn vector_search_returns_rows() {
    let test = create_test_table().await;
    let table = &test.table;
    let fallback_schema = SchemaDefinition::from_arrow_schema(
        table.schema().await.expect("schema").as_ref(),
    );
    let query = table
        .query()
        .nearest_to(vec![0.1_f32, 0.2, 0.3])
        .expect("nearest_to")
        .limit(2);
    let (rows, _) = execute_query_json(query, fallback_schema)
        .await
        .expect("execute query");
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
async fn fts_search_returns_rows() {
    let test = create_test_table().await;
    let table = &test.table;
    table
        .create_index(&["text"], Index::FTS(Default::default()))
        .execute()
        .await
        .expect("create fts index");

    let fallback_schema = SchemaDefinition::from_arrow_schema(
        table.schema().await.expect("schema").as_ref(),
    );
    let query = table
        .query()
        .full_text_search(FullTextSearchQuery::new("alpha".to_string()))
        .limit(5);
    let (rows, _) = execute_query_json(query, fallback_schema)
        .await
        .expect("execute query");
    assert!(!rows.is_empty());
}

#[tokio::test]
async fn commands_happy_path_and_errors() {
    let harness = create_command_harness().await;

    let listed = list_tables_v1_impl(
        &harness.state,
        ListTablesRequestV1 {
            connection_id: harness.connection_id.clone(),
        },
    )
    .await;
    assert!(listed.ok);
    assert_eq!(listed.data.expect("tables").tables.len(), 1);

    let schema = get_schema_v1_impl(
        &harness.state,
        GetSchemaRequestV1 {
            table_id: harness.table_id.clone(),
        },
    )
    .await;
    assert!(schema.ok);
    assert!(
        schema.data.expect("schema").fields.iter().any(|f| f.name == "id"),
        "schema should contain id field"
    );

    // scan json pagination
    let scan_page1 = scan_v1_impl(
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
    assert!(scan_page1.ok);
    let scan_page1 = scan_page1.data.expect("scan data");
    assert_eq!(scan_page1.next_offset, Some(2));
    match scan_page1.chunk {
        DataChunk::Json(chunk) => assert_eq!(chunk.rows.len(), 2),
        _ => panic!("expected json chunk"),
    }

    let scan_last = scan_v1_impl(
        &harness.state,
        ScanRequestV1 {
            table_id: harness.table_id.clone(),
            format: DataFormat::Json,
            projection: None,
            filter: None,
            limit: Some(2),
            offset: Some(4),
        },
    )
    .await;
    assert!(scan_last.ok);
    let scan_last = scan_last.data.expect("scan data");
    assert_eq!(scan_last.next_offset, None);
    match scan_last.chunk {
        DataChunk::Json(chunk) => assert_eq!(chunk.rows.len(), 1),
        _ => panic!("expected json chunk"),
    }

    // scan arrow
    let scan_arrow = scan_v1_impl(
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
    assert!(scan_arrow.ok);
    let scan_arrow = scan_arrow.data.expect("scan arrow");
    assert_eq!(scan_arrow.next_offset, Some(3));

    let ipc_base64 = match scan_arrow.chunk {
        DataChunk::Arrow(chunk) => chunk.ipc_base64,
        _ => panic!("expected arrow chunk"),
    };

    let decoded = general_purpose::STANDARD
        .decode(ipc_base64)
        .expect("decode base64");
    let reader = StreamReader::try_new(Cursor::new(decoded), None).expect("open stream reader");
    let mut row_count = 0;
    for batch in reader {
        let batch = batch.expect("read batch");
        row_count += batch.num_rows();
    }
    assert_eq!(row_count, 3);

    // query filter
    let filtered = query_filter_v1_impl(
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
    assert!(filtered.ok);

    let filtered_empty = query_filter_v1_impl(
        &harness.state,
        QueryFilterRequestV1 {
            table_id: harness.table_id.clone(),
            filter: "  ".to_string(),
            projection: None,
            limit: None,
            offset: None,
        },
    )
    .await;
    assert!(!filtered_empty.ok);
    assert_eq!(
        filtered_empty.error.expect("error").code,
        ErrorCode::InvalidArgument
    );

    // vector search
    let vector_ok = vector_search_v1_impl(
        &harness.state,
        VectorSearchRequestV1 {
            table_id: harness.table_id.clone(),
            vector: vec![0.1, 0.2, 0.3],
            column: None,
            top_k: Some(2),
            projection: None,
            filter: None,
            nprobes: None,
            refine_factor: None,
            offset: Some(0),
        },
    )
    .await;
    assert!(vector_ok.ok);

    let vector_empty = vector_search_v1_impl(
        &harness.state,
        VectorSearchRequestV1 {
            table_id: harness.table_id.clone(),
            vector: vec![],
            column: None,
            top_k: None,
            projection: None,
            filter: None,
            nprobes: None,
            refine_factor: None,
            offset: None,
        },
    )
    .await;
    assert!(!vector_empty.ok);
    assert_eq!(
        vector_empty.error.expect("error").code,
        ErrorCode::InvalidArgument
    );

    // fts search (needs index)
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

    let fts_ok = fts_search_v1_impl(
        &harness.state,
        FtsSearchRequestV1 {
            table_id: harness.table_id.clone(),
            query: "alpha".to_string(),
            columns: None,
            limit: Some(10),
            offset: Some(0),
            projection: None,
            filter: None,
        },
    )
    .await;
    assert!(fts_ok.ok);

    let fts_empty = fts_search_v1_impl(
        &harness.state,
        FtsSearchRequestV1 {
            table_id: harness.table_id.clone(),
            query: " ".to_string(),
            columns: None,
            limit: None,
            offset: None,
            projection: None,
            filter: None,
        },
    )
    .await;
    assert!(!fts_empty.ok);
    assert_eq!(fts_empty.error.expect("error").code, ErrorCode::InvalidArgument);

    // not found
    let missing_schema = get_schema_v1_impl(
        &harness.state,
        GetSchemaRequestV1 {
            table_id: "missing".to_string(),
        },
    )
    .await;
    assert!(!missing_schema.ok);
    assert_eq!(missing_schema.error.expect("error").code, ErrorCode::NotFound);
}

#[tokio::test]
async fn commands_return_internal_when_mutex_poisoned() {
    let state = AppState::new();
    poison_connections_mutex(&state);

    let result = list_tables_v1_impl(
        &state,
        ListTablesRequestV1 {
            connection_id: "any".to_string(),
        },
    )
    .await;

    assert!(!result.ok);
    assert_eq!(result.error.expect("error").code, ErrorCode::Internal);
}
