use gloo_net::http::Request;

use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnqueueRequest {
    pub task_type: TaskTypeRequest,
    pub priority: i32,
    pub context: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TaskTypeRequest {
    Research,
    Coding,
    CreativeWriting,
    SystemOptimization,
    RunCommand,
}

pub fn enqueue_ai_task(task_type: TaskTypeRequest, context: String) {
    spawn_local(async move {
        let req_body = EnqueueRequest {
            task_type,
            priority: 1, // Default priority
            context,
        };

        match Request::post("/api/autonomous/enqueue").json(&req_body) {
            Ok(req) => {
                if let Err(e) = req.send().await {
                    log::error!("Failed to enqueue task: {:?}", e);
                } else {
                    log::info!("AI Task Enqueued Successfully");
                }
            }
            Err(e) => {
                log::error!("Failed to create request: {:?}", e);
            }
        }
    });
}
