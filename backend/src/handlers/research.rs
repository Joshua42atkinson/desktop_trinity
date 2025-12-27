use crate::error::Result;
use crate::game::components::{ResearchLog, VirtueTopology};
use crate::AppState;
use axum::{extract::State, Json};

pub async fn get_research_log(State(state): State<AppState>) -> Result<Json<ResearchLog>> {
    let log = state.shared_research_log.read().unwrap().clone();
    Ok(Json(log))
}

pub async fn get_virtue_topology(State(state): State<AppState>) -> Result<Json<VirtueTopology>> {
    let virtues = *state.shared_virtues.read().unwrap();
    Ok(Json(virtues))
}
