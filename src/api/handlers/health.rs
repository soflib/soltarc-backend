use axum::Json;
use serde_json::{json, Value};

/// Verifica que el servidor está activo
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Servidor activo")
    ),
    tag = "Sistema"
)]
pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}