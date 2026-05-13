// Rutas:
//   POST /auth/token/refresh  → refresh_token
//   POST /auth/token/validate → validate_token

use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::debug;

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{RefreshRequest, ValidateRequest};
use super::grpc_to_http;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshInput {
    pub refresh_jti: String,
    pub device_hint: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ValidateInput {
    pub access_token: String,
}

// ── Refresh token ─────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/token/refresh",
    request_body = RefreshInput,
    responses(
        (status = 200, description = "Token refreshed",  body = Value),
        (status = 401, description = "Invalid token",    body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(body): Json<RefreshInput>,
) -> (StatusCode, Json<Value>) {
    debug!(refresh_jti = %body.refresh_jti, "POST /auth/token/refresh");

    let req = RefreshRequest {
        refresh_jti: body.refresh_jti,
        device_hint: body.device_hint.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.refresh_token(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({
                "access_token":      r.access_token,
                "refresh_jti":       r.refresh_jti,
                "expires_in":        r.expires_in,
                "db_connection_url": r.db_connection_url,
                "user": r.user.map(|u| json!({
                    "user_id":   u.user_id,
                    "email":     u.email,
                    "username":  u.username,
                    "role":      u.role,
                    "status":    u.status,
                    "tenant_id": u.tenant_id,
                })),
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Validate token ────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/token/validate",
    request_body = ValidateInput,
    responses(
        (status = 200, description = "Validation result", body = Value),
        (status = 500, description = "Internal error",    body = Value),
    ),
    tag = "Auth"
)]
pub async fn validate_token(
    State(state): State<AppState>,
    Json(body): Json<ValidateInput>,
) -> (StatusCode, Json<Value>) {
    debug!("POST /auth/token/validate");

    let req = ValidateRequest { access_token: body.access_token };

    let mut client = state.auth_grpc;
    match client.validate_token(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({
                "valid": r.valid,
                "error": r.error,
                "user": r.user.map(|u| json!({
                    "user_id":   u.user_id,
                    "email":     u.email,
                    "username":  u.username,
                    "role":      u.role,
                    "status":    u.status,
                    "tenant_id": u.tenant_id,
                })),
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}
