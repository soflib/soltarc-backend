// Rutas:
//   POST /auth/register  → register
//   POST /auth/login     → login
//   POST /auth/logout    → logout
//   GET  /auth/me        → me

use axum::{extract::{Extension, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, info};

use crate::api::middleware::roles::AuthUser;

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{RegisterRequest, LoginRequest, LogoutRequest};
use super::grpc_to_http;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterInput {
    pub email:          String,
    pub username:       String,
    pub password:       String,
    pub full_name:      Option<String>,
    pub phone:          Option<String>,
    pub role:           Option<String>,
    #[serde(default)]
    #[schema(default = false)]
    pub privat_db:      bool,
    pub tenant_id:      Option<String>,
    pub payment_id:     Option<String>,
    pub payment_plan:   Option<String>,
    pub payment_method: Option<String>,
    pub billing_period: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginInput {
    pub email:       String,
    pub password:    String,
    pub device_hint: Option<String>,
    pub ip_address:  Option<String>,
    pub user_agent:  Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutInput {
    pub refresh_jti:  String,
    pub access_token: Option<String>,
}

// ── Register ─────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterInput,
    responses(
        (status = 200, description = "User registered",  body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterInput>,
) -> (StatusCode, Json<Value>) {
    debug!(email = %body.email, "POST /auth/register");

    let req = RegisterRequest {
        email:          body.email.clone(),
        username:       body.username,
        password:       body.password,
        full_name:      body.full_name.unwrap_or_default(),
        phone:          body.phone.unwrap_or_default(),
        role:           body.role.unwrap_or_default(),
        privat_db:      body.privat_db,
        tenant_id:      body.tenant_id.unwrap_or_default(),
        payment_id:     body.payment_id.unwrap_or_default(),
        payment_plan:   body.payment_plan.unwrap_or_default(),
        payment_method: body.payment_method.unwrap_or_default(),
        billing_period: body.billing_period.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.register(req).await {
        Ok(r) => {
            info!(email = %body.email, "POST /auth/register ← 200");
            (StatusCode::OK, Json(json!({
                "access_token": r.access_token,
                "refresh_jti":  r.refresh_jti,
                "expires_in":   r.expires_in,
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

// ── Login ─────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginInput,
    responses(
        (status = 200, description = "Login successful",     body = Value),
        (status = 401, description = "Invalid credentials",  body = Value),
        (status = 500, description = "Internal error",       body = Value),
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginInput>,
) -> (StatusCode, Json<Value>) {
    debug!(email = %body.email, "POST /auth/login");

    let req = LoginRequest {
        email:       body.email.clone(),
        password:    body.password,
        device_hint: body.device_hint.unwrap_or_default(),
        ip_address:  body.ip_address.unwrap_or_default(),
        user_agent:  body.user_agent.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.login(req).await {
        Ok(r) => {
            info!(email = %body.email, "POST /auth/login ← 200");
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

// ── Logout ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = LogoutInput,
    responses(
        (status = 200, description = "Logged out",       body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<LogoutInput>,
) -> (StatusCode, Json<Value>) {
    debug!(refresh_jti = %body.refresh_jti, "POST /auth/logout");

    let req = LogoutRequest {
        refresh_jti:  body.refresh_jti,
        access_token: body.access_token.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.logout(req).await {
        Ok(r) => {
            info!("POST /auth/logout ← 200 success={}", r.success);
            (StatusCode::OK, Json(json!({ "success": r.success })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Me ────────────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user info", body = Value),
        (status = 401, description = "Not authenticated", body = Value),
    ),
    tag = "Auth"
)]
pub async fn me(
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!(username = %auth_user.username, "GET /auth/me");
    (StatusCode::OK, Json(json!({
        "user_id":   auth_user.user_id,
        "username":  auth_user.username,
        "role":      auth_user.role,
        "roles":     [auth_user.role],
        "tenant_id": auth_user.tenant_id,
    })))
}
