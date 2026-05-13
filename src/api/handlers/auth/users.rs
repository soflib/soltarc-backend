// Routes:
//   GET    /auth/users              → get_all_users  (?limit=&offset=&tenant_id=)
//   POST   /auth/users              → create_user
//   GET    /auth/users/check        → check_username  (?username=)
//   GET    /auth/users/{id}         → get_user
//   DELETE /auth/users/{id}         → delete_user
//   PUT    /auth/users/{id}         → update_user
//   PUT    /auth/users/{id}/lock    → lock_user

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;
use utoipa::ToSchema;

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{
    GetAllUsersRequest, GetUserRequest, DeleteUserRequest,
    UpdateUserRequest, LockUserRequest, CheckUsernameRequest,
    RegisterRequest,
};
use super::grpc_to_http;

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct GetAllUsersQuery {
    pub limit:     Option<i32>,
    pub offset:    Option<i32>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateUserInput {
    pub email:     String,
    pub username:  Option<String>,
    pub password:  String,
    pub full_name: Option<String>,
    pub phone:     Option<String>,
    pub role:      String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct UpdateUserInput {
    pub full_name: Option<String>,
    pub phone:     Option<String>,
    pub role:      Option<String>,
    pub status:    Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LockUserInput {
    pub lock:    bool,
    pub minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CheckUsernameQuery {
    pub username: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/users",
    params(
        ("limit"     = Option<i32>,    Query, description = "Max records (default 50)"),
        ("offset"    = Option<i32>,    Query, description = "Records to skip (default 0)"),
        ("tenant_id" = Option<String>, Query, description = "Filter by tenant"),
    ),
    responses(
        (status = 200, description = "List of users",  body = Value),
        (status = 500, description = "Internal error", body = Value),
    ),
    tag = "Auth"
)]
pub async fn get_all_users(
    State(state): State<AppState>,
    Query(q): Query<GetAllUsersQuery>,
) -> (StatusCode, Json<Value>) {
    let limit  = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    debug!(limit, offset, "GET /auth/users");

    let req = GetAllUsersRequest {
        limit,
        offset,
        tenant_id: q.tenant_id.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.get_all_users(req).await {
        Ok(r) => {
            let users: Vec<Value> = r.users.into_iter().map(|u| json!({
                "user_id":      u.user_id,
                "email":        u.email,
                "username":     u.username,
                "full_name":    u.full_name,
                "role":         u.role,
                "status":       u.status,
                "tenant_id":    u.tenant_id,
                "locked_until": u.locked_until,
            })).collect();
            (StatusCode::OK, Json(json!({ "users": users, "total": r.total })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/users",
    request_body = CreateUserInput,
    responses(
        (status = 201, description = "User created",       body = Value),
        (status = 409, description = "Email/username taken", body = Value),
        (status = 500, description = "Internal error",     body = Value),
    ),
    tag = "Auth"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CreateUserInput>,
) -> (StatusCode, Json<Value>) {
    debug!(email = %body.email, "POST /auth/users");

    if auth_user.tenant_id.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "caller has no tenant" })));
    }

    let req = RegisterRequest {
        email:          body.email,
        username:       body.username.unwrap_or_default(),
        password:       body.password,
        full_name:      body.full_name.unwrap_or_default(),
        phone:          body.phone.unwrap_or_default(),
        role:           body.role,
        privat_db:      false,
        payment_id:     String::new(),
        tenant_id:      auth_user.tenant_id,
        payment_plan:   String::new(),
        payment_method: String::new(),
        billing_period: String::new(),
    };

    let mut client = state.auth_grpc;
    match client.register(req).await {
        Ok(_) => (StatusCode::CREATED, Json(json!({ "success": true }))),
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    get,
    path = "/auth/users/check",
    params(("username" = String, Query, description = "Username to check")),
    responses(
        (status = 200, description = "Availability result", body = Value),
    ),
    tag = "Auth"
)]
pub async fn check_username(
    State(state): State<AppState>,
    Query(q): Query<CheckUsernameQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(username = %q.username, "GET /auth/users/check");

    let req = CheckUsernameRequest { username: q.username };
    let mut client = state.auth_grpc;
    match client.check_username(req).await {
        Ok(r) => (StatusCode::OK, Json(json!({ "available": r.available }))),
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    get,
    path = "/auth/users/{id}",
    params(("id" = String, Path, description = "User UUID")),
    responses(
        (status = 200, description = "User detail",    body = Value),
        (status = 404, description = "User not found", body = Value),
    ),
    tag = "Auth"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %id, "GET /auth/users/:id");

    let req = GetUserRequest { user_id: id };
    let mut client = state.auth_grpc;
    match client.get_user(req).await {
        Ok(u) => (StatusCode::OK, Json(json!({
            "user_id":         u.user_id,
            "email":           u.email,
            "username":        u.username,
            "full_name":       u.full_name,
            "phone":           u.phone,
            "role":            u.role,
            "status":          u.status,
            "tenant_id":       u.tenant_id,
            "last_login_at":   u.last_login_at,
            "failed_attempts": u.failed_attempts,
            "locked_until":    u.locked_until,
            "created_at":      u.created_at,
        }))),
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/auth/users/{id}",
    params(("id" = String, Path, description = "User UUID")),
    responses(
        (status = 200, description = "Deleted",        body = Value),
        (status = 404, description = "User not found", body = Value),
    ),
    tag = "Auth"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %id, "DELETE /auth/users/:id");

    let req = DeleteUserRequest { user_id: id };
    let mut client = state.auth_grpc;
    match client.delete_user(req).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    put,
    path = "/auth/users/{id}",
    params(("id" = String, Path, description = "User UUID")),
    request_body = UpdateUserInput,
    responses(
        (status = 200, description = "Updated user",   body = Value),
        (status = 404, description = "User not found", body = Value),
    ),
    tag = "Auth"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserInput>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %id, "PUT /auth/users/:id");

    let req = UpdateUserRequest {
        user_id:   id,
        full_name: body.full_name.unwrap_or_default(),
        phone:     body.phone.unwrap_or_default(),
        role:      body.role.unwrap_or_default(),
        status:    body.status.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.update_user(req).await {
        Ok(u) => (StatusCode::OK, Json(json!({
            "user_id":   u.user_id,
            "email":     u.email,
            "username":  u.username,
            "full_name": u.full_name,
            "phone":     u.phone,
            "role":      u.role,
            "status":    u.status,
        }))),
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

#[utoipa::path(
    put,
    path = "/auth/users/{id}/lock",
    params(("id" = String, Path, description = "User UUID")),
    request_body = LockUserInput,
    responses(
        (status = 200, description = "Lock toggled",   body = Value),
        (status = 404, description = "User not found", body = Value),
    ),
    tag = "Auth"
)]
pub async fn lock_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<LockUserInput>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %id, lock = body.lock, "PUT /auth/users/:id/lock");

    let req = LockUserRequest {
        user_id: id,
        lock:    body.lock,
        minutes: body.minutes.unwrap_or(0),
    };

    let mut client = state.auth_grpc;
    match client.lock_user(req).await {
        Ok(r) => (StatusCode::OK, Json(json!({
            "success":      r.success,
            "locked_until": r.locked_until,
        }))),
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}
