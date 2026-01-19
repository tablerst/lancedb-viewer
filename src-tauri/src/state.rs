use std::sync::Mutex;

use crate::services::connection_manager::ConnectionManager;

pub struct AppState {
    pub connections: Mutex<ConnectionManager>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(ConnectionManager::new()),
        }
    }
}
