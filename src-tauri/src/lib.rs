mod db;
// mod sync;
// mod ai;

use crate::db::JournalStore;
// use crate::sync::SyncService;
// use crate::ai::SocraticAgent;
use std::sync::Mutex;
use tauri::{Manager, State};

/*
pub struct SyncService {
    backend_url: String,
}

impl SyncService {
    pub fn new(backend_url: String) -> Self {
        Self {
            backend_url,
        }
    }

    pub async fn push_xp(&self, _amount: i32, _reason: &str) -> std::result::Result<(), String> {
        Ok(())
    }
}
*/

struct AppState {
    store: Mutex<JournalStore>,
    // sync: SyncService,
    // agent: Mutex<Option<SocraticAgent>>,
}

#[tauri::command]
async fn submit_journal(state: State<'_, AppState>, content: String) -> std::result::Result<String, String> {
    // 1. Save to DB
    let mut store = state.store.lock().map_err(|e| e.to_string())?;
    let entry_id = store.add_entry(&content, "").map_err(|e| e.to_string())?;

    // 4. Award XP (Mock logic)
    // let _ = state.sync.push_xp(50, "Journal Entry").await;

    Ok("Entry Saved".to_string())
}

/*
#[tauri::command]
async fn sync_xp(state: State<'_, AppState>) -> std::result::Result<String, String> {
    state
        .sync
        .push_xp(0, "Manual Sync")
        .await
        .map_err(|e| e.to_string())?;
    Ok("Synced".to_string())
}
*/

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(AppState {
            store: Mutex::new(
                JournalStore::init("journal.db", "secret_key").expect("Failed to init DB"),
            ),
            // sync: SyncService::new("http://192.168.2.141:3000".to_string()),
        })
        .invoke_handler(tauri::generate_handler![submit_journal])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
