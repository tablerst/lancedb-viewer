use std::collections::HashMap;

use lancedb::{Connection, Table};
use uuid::Uuid;

#[derive(Default)]
pub struct ConnectionManager {
    connections: HashMap<String, Connection>,
    tables: HashMap<String, StoredTable>,
}

#[derive(Clone)]
struct StoredTable {
    name: String,
    table: Table,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_connection(&mut self, connection: Connection) -> String {
        let id = Uuid::new_v4().to_string();
        self.connections.insert(id.clone(), connection);
        id
    }

    pub fn get_connection(&self, connection_id: &str) -> Option<Connection> {
        self.connections.get(connection_id).cloned()
    }

    pub fn insert_table(&mut self, name: String, table: Table) -> String {
        let id = Uuid::new_v4().to_string();
        self.tables.insert(id.clone(), StoredTable { name, table });
        id
    }

    pub fn get_table(&self, table_id: &str) -> Option<Table> {
        self.tables.get(table_id).map(|entry| entry.table.clone())
    }

    pub fn get_table_name(&self, table_id: &str) -> Option<String> {
        self.tables.get(table_id).map(|entry| entry.name.clone())
    }
}
