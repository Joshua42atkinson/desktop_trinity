use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct XpUpdate {
    pub amount: i32,
    pub reason: String,
}

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
