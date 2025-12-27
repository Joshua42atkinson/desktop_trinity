use crate::{AppError, AppState, Result};
use axum::{extract::State, Json};
use common::expert::StoryGraph;
use sqlx::Row;

pub async fn get_graph(State(app_state): State<AppState>) -> Result<Json<StoryGraph>> {
    // Check if we are in simulation mode (no DB)
    let pool = match app_state.pool {
        Some(pool) => pool,
        None => {
            // Return a default graph in simulation mode
            let default_graph = StoryGraph {
                id: "demo_graph".to_string(),
                title: "Simulation Story".to_string(),
                nodes: vec![],
                connections: vec![],
            };
            return Ok(Json(default_graph));
        }
    };

    // Fetch the graph (hardcoded ID for now, similar to mock)
    let row = sqlx::query("SELECT nodes, connections, title FROM story_graphs WHERE id = $1")
        .bind("demo_graph")
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            AppError::InternalServerError(e.to_string())
        })?;

    if let Some(row) = row {
        let nodes: serde_json::Value = row.get("nodes");
        let connections: serde_json::Value = row.get("connections");
        let title: String = row.get("title");

        let graph = StoryGraph {
            id: "demo_graph".to_string(),
            title,
            nodes: serde_json::from_value(nodes).unwrap_or_default(),
            connections: serde_json::from_value(connections).unwrap_or_default(),
        };
        Ok(Json(graph))
    } else {
        // Return a default empty graph if not found (auto-create logic could go here)
        let default_graph = StoryGraph {
            id: "demo_graph".to_string(),
            title: "New Story".to_string(),
            nodes: vec![],
            connections: vec![],
        };
        Ok(Json(default_graph))
    }
}

pub async fn save_graph(
    State(app_state): State<AppState>,
    Json(payload): Json<StoryGraph>,
) -> Result<Json<StoryGraph>> {
    let pool = match app_state.pool {
        Some(pool) => pool,
        None => {
            // Mock save in simulation mode
            return Ok(Json(payload));
        }
    };

    let nodes_json = serde_json::to_value(&payload.nodes).unwrap();
    let connections_json = serde_json::to_value(&payload.connections).unwrap();

    sqlx::query(
        r#"
        INSERT INTO story_graphs (id, title, nodes, connections, updated_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (id) DO UPDATE
        SET title = EXCLUDED.title,
            nodes = EXCLUDED.nodes,
            connections = EXCLUDED.connections,
            updated_at = NOW()
        "#,
    )
    .bind(&payload.id)
    .bind(&payload.title)
    .bind(nodes_json)
    .bind(connections_json)
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {:?}", e);
        AppError::InternalServerError(e.to_string())
    })?;

    Ok(Json(payload))
}
