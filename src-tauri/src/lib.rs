mod commands;
mod domain;
pub mod ipc;
pub mod services;
pub mod state;

use log::LevelFilter;
use sha2::{Digest, Sha256};
use tauri_plugin_log::{Target, TargetKind};

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_level = if cfg!(debug_assertions) {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("lancedb-viewer.log".to_string()),
                    }),
                ])
                .level(log_level)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_stronghold::Builder::new(|password| {
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                hasher.finalize().to_vec()
            })
            .build(),
        )
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::v1::connect_v1,
            commands::v1::disconnect_v1,
            commands::v1::list_tables_v1,
            commands::v1::drop_table_v1,
            commands::v1::rename_table_v1,
            commands::v1::list_indexes_v1,
            commands::v1::create_index_v1,
            commands::v1::drop_index_v1,
            commands::v1::create_table_v1,
            commands::v1::open_table_v1,
            commands::v1::get_schema_v1,
            commands::v1::list_versions_v1,
            commands::v1::get_table_version_v1,
            commands::v1::checkout_table_version_v1,
            commands::v1::checkout_table_latest_v1,
            commands::v1::clone_table_v1,
            commands::v1::add_columns_v1,
            commands::v1::alter_columns_v1,
            commands::v1::drop_columns_v1,
            commands::v1::write_rows_v1,
            commands::v1::update_rows_v1,
            commands::v1::delete_rows_v1,
            commands::v1::import_data_v1,
            commands::v1::export_data_v1,
            commands::v1::optimize_table_v1,
            commands::v1::scan_v1,
            commands::v1::query_filter_v1,
            commands::v1::combined_search_v1,
            commands::v1::vector_search_v1,
            commands::v1::fts_search_v1,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
