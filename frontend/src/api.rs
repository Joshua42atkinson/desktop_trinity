use leptos::prelude::ServerFnError;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectionData {
    pub archetype: String,
    pub virtue_focus: String,
    pub dilemma_choice: String, // "A", "B", "C", or "D"
}

#[server]
pub async fn submit_reflection(data: ReflectionData) -> Result<(), ServerFnError> {
    // Here, we would bridge to Bevy to spawn the entity.
    // let world =... (Access Bevy World);
    println!("Received Reflection: {:?}", data);
    Ok(())
}

// --- AI Mirror API Client ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendMessageRequest {
    pub session_id: Uuid,
    pub user_id: i64,
    pub message: String,
    pub archetype: Option<String>,
    pub focus_area: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendMessageResponse {
    pub ai_response: String,
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
}

pub async fn create_session() -> Result<Uuid, String> {
    let res = gloo_net::http::Request::post("http://localhost:3000/api/ai-mirror/create-session")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.ok() {
        let body: CreateSessionResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(body.session_id)
    } else {
        Err(format!("Failed to create session: {}", res.status()))
    }
}

pub async fn send_message(req: SendMessageRequest) -> Result<SendMessageResponse, String> {
    let res = gloo_net::http::Request::post("http://localhost:3000/api/ai-mirror/send-message")
        .json(&req)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.ok() {
        let body: SendMessageResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(body)
    } else {
        Err(format!("Failed to send message: {}", res.status()))
    }
}

// --- Expert Module API ---

use common::expert::StoryGraph;

pub async fn get_graph() -> Result<StoryGraph, String> {
    let res = gloo_net::http::Request::get("http://localhost:3000/api/expert/graph")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.ok() {
        let graph: StoryGraph = res.json().await.map_err(|e| e.to_string())?;
        Ok(graph)
    } else {
        Err(format!("Failed to fetch graph: {}", res.status()))
    }
}

pub async fn save_graph(graph: StoryGraph) -> Result<StoryGraph, String> {
    let res = gloo_net::http::Request::post("http://localhost:3000/api/expert/graph")
        .json(&graph)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.ok() {
        let saved_graph: StoryGraph = res.json().await.map_err(|e| e.to_string())?;
        Ok(saved_graph)
    } else {
        Err(format!("Failed to save graph: {}", res.status()))
    }
}
