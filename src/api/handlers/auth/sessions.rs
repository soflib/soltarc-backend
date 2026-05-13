// Rutas:
//   POST /auth/sessions/revoke            → revoke_sessions
//   PUT  /auth/password                   → change_password
//   GET  /auth/password/random?length=N   → random_password

use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::debug;

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{RevokeSessionsRequest, ChangePasswordRequest};
use super::grpc_to_http;

// ── Random-password charset ───────────────────────────────────────────────────
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%&*-_";

fn generate_password(length: usize) -> String {
    let mut out = String::with_capacity(length);
    while out.len() < length {
        for &b in uuid::Uuid::new_v4().as_bytes() {
            if out.len() < length {
                out.push(CHARSET[b as usize % CHARSET.len()] as char);
            }
        }
    }
    out
}

#[derive(Debug, Deserialize)]
pub struct RandomPasswordQuery {
    pub length: Option<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RevokeSessionsInput {
    pub user_id:         String,
    pub include_current: Option<bool>,
    pub current_jti:     Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordInput {
    pub user_id:          String,
    pub current_password: String,
    pub new_password:     String,
    pub revoke_sessions:  Option<bool>,
}

// ── Revoke sessions ───────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/sessions/revoke",
    request_body = RevokeSessionsInput,
    responses(
        (status = 200, description = "Sessions revoked", body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn revoke_sessions(
    State(state): State<AppState>,
    Json(body): Json<RevokeSessionsInput>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %body.user_id, "POST /auth/sessions/revoke");

    let req = RevokeSessionsRequest {
        user_id:         body.user_id,
        include_current: body.include_current.unwrap_or(false),
        current_jti:     body.current_jti.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.revoke_sessions(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({ "sessions_revoked": r.sessions_revoked })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Change password ───────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/auth/password",
    request_body = ChangePasswordInput,
    responses(
        (status = 200, description = "Password changed", body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 401, description = "Unauthorized",     body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn change_password(
    State(state): State<AppState>,
    Json(body): Json<ChangePasswordInput>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %body.user_id, "PUT /auth/password");

    let req = ChangePasswordRequest {
        user_id:          body.user_id,
        current_password: body.current_password,
        new_password:     body.new_password,
        revoke_sessions:  body.revoke_sessions.unwrap_or(false),
    };

    let mut client = state.auth_grpc;
    match client.change_password(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({
                "success":          r.success,
                "sessions_revoked": r.sessions_revoked,
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Random password ───────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/password/random",
    params(("length" = Option<u8>, Query, description = "Password length (8–64, default 12)")),
    responses(
        (status = 200, description = "Generated password", body = Value),
    ),
    tag = "Auth"
)]
pub async fn random_password(
    Query(q): Query<RandomPasswordQuery>,
) -> (StatusCode, Json<Value>) {
    let length = q.length.unwrap_or(12).clamp(8, 64) as usize;
    debug!(length, "GET /auth/password/random");
    let password = generate_password(length);
    (StatusCode::OK, Json(json!({ "password": password, "length": length })))
}
