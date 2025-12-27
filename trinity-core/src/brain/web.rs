use crate::brain::Brain;
use anyhow::Result;
use async_trait::async_trait;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = "trinityThink")]
    async fn js_think(prompt: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = window, js_name = "trinityLoadModel")]
    async fn js_load_model(model_id: &str) -> JsValue;
}

pub struct WebBrain;

#[async_trait(?Send)]
impl Brain for WebBrain {
    async fn think(&self, prompt: &str) -> Result<String> {
        let result = js_think(prompt).await;
        result
            .as_string()
            .ok_or_else(|| anyhow::anyhow!("JS think returned non-string"))
    }

    async fn load_model(&self, model_id: &str) -> Result<()> {
        let _ = js_load_model(model_id).await;
        Ok(())
    }
}
