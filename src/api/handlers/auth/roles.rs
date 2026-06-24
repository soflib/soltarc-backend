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
use crate::generated::auth::{GetAllUsersRequest, UpdateUserRequest};
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
    // username ya NO es único global → se resuelve siempre DENTRO del tenant del que llama.
    let req = GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: auth_user.tenant_id.clone() };
    match client.get_all_users(req).await {
        Ok(r) => match r.users.into_iter().find(|u| u.username == q.username) {
            Some(u) => (StatusCode::OK, Json(json!({
                "user_id":  u.user_id,
                "username": u.username,
                "roles":    [u.role],
            }))),
            None => (StatusCode::NOT_FOUND, Json(json!({ "error": "user not found in your tenant" }))),
        },
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

    // 1. Buscar el usuario por username DENTRO del tenant del que llama.
    //    username ya NO es único global, así que la búsqueda global ya no aplica:
    //    listamos los usuarios del tenant y filtramos. Esto además garantiza que
    //    solo se pueda asignar rol a usuarios del propio tenant.
    let req = GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: auth_user.tenant_id.clone() };
    let user = match client.get_all_users(req).await {
        Ok(r) => match r.users.into_iter().find(|u| u.username == body.username) {
            Some(u) => u,
            None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "user not found in your tenant" }))),
        },
        Err(status) => {
            let (code, b) = grpc_to_http(status);
            return (code, Json(b));
        }
    };

    // 2. Update only the role field (empty strings = keep existing values)
    match client.update_user(UpdateUserRequest {
        user_id:   user.user_id,
        full_name: String::new(),
        phone:     String::new(),
        role:      body.role,
        status:    String::new(),
        email:     String::new(),
        username:  String::new(),
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
