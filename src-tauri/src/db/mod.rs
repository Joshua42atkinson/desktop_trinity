use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Option<i64>,
    pub content: String,
    pub reflection: String,
    pub timestamp: i64,
}

pub struct JournalStore {
    conn: Connection,
}

impl JournalStore {
    pub fn init<P: AsRef<Path>>(path: P, key: &str) -> Result<Self> {
        let conn = Connection::open(path)?;

        // Set the encryption key (SQLCipher)
        conn.pragma_update(None, "key", &key)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS journal (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL,
                reflection TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(JournalStore { conn })
    }

    pub fn add_entry(&self, content: &str, reflection: &str) -> Result<i64> {
        let timestamp = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO journal (content, reflection, timestamp) VALUES (?1, ?2, ?3)",
            (content, reflection, timestamp),
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_entries(&self) -> Result<Vec<JournalEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content, reflection, timestamp FROM journal ORDER BY timestamp DESC",
        )?;
        let entry_iter = stmt.query_map([], |row| {
            Ok(JournalEntry {
                id: Some(row.get(0)?),
                content: row.get(1)?,
                reflection: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for entry in entry_iter {
            entries.push(entry?);
        }
        Ok(entries)
    }
}
