use crate::ai::llm::gguf_loader::ModelType;
use crate::ai::llm::{GgufConfig, GgufModel};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global manager for loaded LLM models to prevent reloading and OOM
#[derive(Clone)]
pub struct ModelManager {
    models: Arc<Mutex<HashMap<String, Arc<Mutex<GgufModel>>>>>,
}

impl ModelManager {
    pub fn new() -> Self {
        Self {
            models: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get or load a model of the specified type
    pub fn get_model(&self, model_type: ModelType) -> Result<Arc<Mutex<GgufModel>>> {
        let key = format!("{:?}", model_type);
        let mut models = self.models.lock().unwrap();

        if let Some(model) = models.get(&key) {
            return Ok(model.clone());
        }

        // Load new model
        log::info!("ModelManager: Loading {:?} model...", model_type);
        let config = GgufConfig::from_env(model_type);
        let model = GgufModel::load(config)?;
        let model_ref = Arc::new(Mutex::new(model));

        models.insert(key, model_ref.clone());
        Ok(model_ref)
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}
