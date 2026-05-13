// Routes:
//   GET  /auth/roles                → list_roles       (static list)
//   GET  /auth/roles/user           → get_user_roles   (?username=)
//   POST /auth/roles/assign         → assign_role      (body: {username, role})

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;
use utoipa::ToSchema;

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{GetByUsernameRequest, UpdateUserRequest};
use super::grpc_to_http;

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetUserRolesQuery {
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AssignRoleInput {
    pub username: String,
    pub role:     String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/roles",
    responses(
        (status = 200, description = "Available roles", body = Value),
    ),
    tag = "Auth"
)]
pub async fn list_roles() -> (StatusCode, Json<Value>) {
    (StatusCode::OK, Json(json!({
        "roles": ["admin", "arquitecto", "finanzas", "reportes"]
    })))
}

#[utoipa::path(
    get,
    path = "/auth/roles/user",
    params(("username" = String, Query, description = "Username to look up")),
    responses(
        (status = 200, description = "User's current role",  body = Value),
        (status = 403, description = "User not in tenant",   body = Value),
        (status = 404, description = "User not found",       body = Value),
    ),
    tag = "Auth"
)]
pub async fn get_user_roles(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<GetUserRolesQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(username = %q.username, "GET /auth/roles/user");

    let mut client = state.auth_grpc;
    match client.get_user_by_username(GetByUsernameRequest { username: q.username }).await {
        Ok(u) => {
            if u.tenant_id != auth_user.tenant_id {
                return (StatusCode::FORBIDDEN, Json(json!({ "error": "user not in your tenant" })));
            }
            (StatusCode::OK, Json(json!({
                "user_id":  u.user_id,
                "username": u.username,
                "roles":    [u.role],
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/roles/assign",
    request_body = AssignRoleInput,
    responses(
        (status = 200, description = "Role assigned",        body = Value),
        (status = 403, description = "User not in tenant",   body = Value),
        (status = 404, description = "User not found",       body = Value),
    ),
    tag = "Auth"
)]
pub async fn assign_role(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<AssignRoleInput>,
) -> (StatusCode, Json<Value>) {
    debug!(username = %body.username, role = %body.role, "POST /auth/roles/assign");

    let mut client = state.auth_grpc;

    // 1. Look up user by username
    let user = match client.get_user_by_username(GetByUsernameRequest { username: body.username.clone() }).await {
        Ok(u) => u,
        Err(status) => {
            let (code, b) = grpc_to_http(status);
            return (code, Json(b));
        }
    };

    // 2. Verify the user belongs to the caller's tenant
    if user.tenant_id != auth_user.tenant_id {
        return (StatusCode::FORBIDDEN, Json(json!({ "error": "user not in your tenant" })));
    }

    // 3. Update only the role field (empty strings = keep existing values)
    match client.update_user(UpdateUserRequest {
        user_id:   user.user_id,
        full_name: String::new(),
        phone:     String::new(),
        role:      body.role,
        status:    String::new(),
    }).await {
        Ok(u) => (StatusCode::OK, Json(json!({
            "user_id":  u.user_id,
            "username": u.username,
            "role":     u.role,
        }))),
        Err(status) => {
            let (code, b) = grpc_to_http(status);
            (code, Json(b))
        }
    }
}
