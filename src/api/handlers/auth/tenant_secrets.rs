// Rutas:
//   PUT /auth/tenants/{id}/db-url → set_tenant_db_url
//   GET /auth/tenants/{id}/db-url → get_tenant_db_url

use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, info};

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{SetTenantDbUrlRequest, GetTenantDbUrlRequest};
use super::grpc_to_http;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetTenantDbUrlInput {
    pub db_connection_url: String,
}

// ── Set tenant DB URL ─────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/auth/tenants/{id}/db-url",
    params(("id" = String, Path, description = "Tenant UUID")),
    request_body = SetTenantDbUrlInput,
    responses(
        (status = 200, description = "DB URL stored",     body = Value),
        (status = 400, description = "Bad request",       body = Value),
        (status = 404, description = "Tenant not found",  body = Value),
        (status = 500, description = "Internal error",    body = Value),
    ),
    tag = "Auth"
)]
pub async fn set_tenant_db_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<SetTenantDbUrlInput>,
) -> (StatusCode, Json<Value>) {
    debug!(tenant_id = %id, "PUT /auth/tenants/{id}/db-url");

    let req = SetTenantDbUrlRequest {
        tenant_id:        id.clone(),
        db_connection_url: body.db_connection_url,
    };

    let mut client = state.auth_grpc;
    match client.set_tenant_db_url(req).await {
        Ok(_) => {
            info!(tenant_id = %id, "tenant DB URL updated");
            (StatusCode::OK, Json(json!({ "success": true })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Get tenant DB URL ─────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/tenants/{id}/db-url",
    params(("id" = String, Path, description = "Tenant UUID")),
    responses(
        (status = 200, description = "DB URL returned",   body = Value),
        (status = 404, description = "Secret not found",  body = Value),
        (status = 500, description = "Internal error",    body = Value),
    ),
    tag = "Auth"
)]
pub async fn get_tenant_db_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    debug!(tenant_id = %id, "GET /auth/tenants/{id}/db-url");

    let req = GetTenantDbUrlRequest { tenant_id: id };

    let mut client = state.auth_grpc;
    match client.get_tenant_db_url(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({ "db_connection_url": r.db_connection_url })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}
