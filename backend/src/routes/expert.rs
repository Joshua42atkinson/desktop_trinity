use crate::handlers::expert::{get_graph, save_graph};
use crate::AppState;
use axum::{routing::get, Router};

pub fn expert_routes(_state: &AppState) -> Router<AppState> {
    Router::new().route("/api/expert/graph", get(get_graph).post(save_graph))
}
