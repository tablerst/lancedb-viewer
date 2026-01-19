mod commands;
mod domain;
mod ipc;
mod services;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::v1::connect_v1,
            commands::v1::list_tables_v1,
            commands::v1::open_table_v1,
            commands::v1::get_schema_v1,
            commands::v1::scan_v1,
            commands::v1::query_filter_v1,
            commands::v1::vector_search_v1,
            commands::v1::fts_search_v1,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
