use std::error::Error;
use std::sync::Arc;

use arrow_array::types::Float32Type;
use arrow_array::{FixedSizeListArray, Int32Array, RecordBatch, RecordBatchIterator, StringArray};
use arrow_schema::{DataType, Field, Schema};

fn main() {
	if let Err(error) = tauri::async_runtime::block_on(run()) {
		eprintln!("seed_db failed: {error}");
		std::process::exit(1);
	}
}

async fn run() -> Result<(), Box<dyn Error>> {
	let args = parse_args()?;
	let (batch, schema) = build_batch(args.rows)?;
	let batches = RecordBatchIterator::new(vec![batch].into_iter().map(Ok), schema);

	let db = lancedb::connect(&args.path).execute().await?;
	let table_names = db.table_names().execute().await?;
	if table_names.iter().any(|name| name == &args.table) {
		println!(
			"table '{}' already exists in {}. Remove it or choose another name.",
			args.table, args.path
		);
		return Ok(());
	}

	db.create_table(&args.table, Box::new(batches))
		.execute()
		.await?;

	println!(
		"Created sample LanceDB at {} (table: {}, rows: {}).",
		args.path, args.table, args.rows
	);
	println!("Use the path above as the URI in the app connection profile.");

	Ok(())
}

struct Args {
	path: String,
	table: String,
	rows: usize,
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
	let mut path: Option<String> = None;
	let mut table = "items".to_string();
	let mut rows = 50usize;

	let mut iter = std::env::args().skip(1).peekable();
	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"-h" | "--help" => {
				print_usage();
				std::process::exit(0);
			}
			"--path" => {
				path = Some(
					iter.next()
						.ok_or("missing value for --path")?
						.to_string(),
				);
			}
			"--table" => {
				table = iter
					.next()
					.ok_or("missing value for --table")?
					.to_string();
			}
			"--rows" => {
				rows = iter
					.next()
					.ok_or("missing value for --rows")?
					.parse()?;
			}
			value if !value.starts_with('-') && path.is_none() => {
				path = Some(value.to_string());
			}
			unknown => {
				return Err(format!("unknown argument: {unknown}").into());
			}
		}
	}

	let path = path.unwrap_or_else(|| "sample-db".to_string());
	let rows = rows.max(1).min(10_000);

	Ok(Args { path, table, rows })
}

fn print_usage() {
	println!("Usage: seed_db [path] [--table <name>] [--rows <count>]");
	println!("Defaults: path=sample-db, table=items, rows=50");
}

fn build_batch(rows: usize) -> Result<(RecordBatch, Arc<Schema>), Box<dyn Error>> {
	let schema = Arc::new(Schema::new(vec![
		Field::new("id", DataType::Int32, false),
		Field::new("text", DataType::Utf8, false),
		Field::new(
			"vector",
			DataType::FixedSizeList(
				Arc::new(Field::new("item", DataType::Float32, true)),
				3,
			),
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
	)?;

	Ok((batch, schema))
}
