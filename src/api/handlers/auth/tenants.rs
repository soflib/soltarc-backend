// Rutas:
//   POST   /auth/tenants      → create_tenant
//   GET    /auth/tenants      → list_tenants   (?limit=&offset=)
//   GET    /auth/tenants/{id} → get_tenant
//   PUT    /auth/tenants/{id} → update_tenant
//   DELETE /auth/tenants/{id} → delete_tenant

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, info};

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::{
    CreateTenantRequest, GetTenantRequest, UpdateTenantRequest,
    DeleteTenantRequest, ListTenantsRequest,
};
use super::grpc_to_http;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTenantInput {
    pub name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTenantInput {
    pub name:   Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
    pub limit:  Option<i32>,
    pub offset: Option<i32>,
}

// ── Create tenant ─────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/auth/tenants",
    request_body = CreateTenantInput,
    responses(
        (status = 201, description = "Tenant created",  body = Value),
        (status = 400, description = "Bad request",     body = Value),
        (status = 500, description = "Internal error",  body = Value),
    ),
    tag = "Auth"
)]
pub async fn create_tenant(
    State(state): State<AppState>,
    Json(body): Json<CreateTenantInput>,
) -> (StatusCode, Json<Value>) {
    debug!(name = %body.name, "POST /auth/tenants");

    let req = CreateTenantRequest { name: body.name };

    let mut client = state.auth_grpc;
    match client.create_tenant(req).await {
        Ok(r) => {
            info!(tenant_id = %r.tenant_id, "POST /auth/tenants ← 201");
            (StatusCode::CREATED, Json(json!({
                "tenant_id":  r.tenant_id,
                "name":       r.name,
                "status":     r.status,
                "created_at": r.created_at,
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── List tenants ──────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/tenants",
    params(
        ("limit"  = Option<i32>, Query, description = "Max records to return (default 50)"),
        ("offset" = Option<i32>, Query, description = "Records to skip (default 0)"),
    ),
    responses(
        (status = 200, description = "List of tenants", body = Value),
        (status = 500, description = "Internal error",  body = Value),
    ),
    tag = "Auth"
)]
pub async fn list_tenants(
    State(state): State<AppState>,
    Query(q): Query<ListTenantsQuery>,
) -> (StatusCode, Json<Value>) {
    let limit  = q.limit.unwrap_or(50);
    let offset = q.offset.unwrap_or(0);
    debug!(limit, offset, "GET /auth/tenants");

    let req = ListTenantsRequest { limit, offset };

    let mut client = state.auth_grpc;
    match client.list_tenants(req).await {
        Ok(r) => {
            let tenants: Vec<Value> = r.tenants.into_iter().map(|t| json!({
                "tenant_id":  t.tenant_id,
                "name":       t.name,
                "status":     t.status,
                "created_at": t.created_at,
            })).collect();

            (StatusCode::OK, Json(json!({ "tenants": tenants, "total": r.total })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Get tenant ────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/auth/tenants/{id}",
    params(("id" = String, Path, description = "Tenant UUID")),
    responses(
        (status = 200, description = "Tenant found",     body = Value),
        (status = 404, description = "Tenant not found", body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn get_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    debug!(tenant_id = %id, "GET /auth/tenants/{id}");

    let req = GetTenantRequest { tenant_id: id };

    let mut client = state.auth_grpc;
    match client.get_tenant(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({
                "tenant_id":  r.tenant_id,
                "name":       r.name,
                "status":     r.status,
                "created_at": r.created_at,
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Update tenant ─────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/auth/tenants/{id}",
    params(("id" = String, Path, description = "Tenant UUID")),
    request_body = UpdateTenantInput,
    responses(
        (status = 200, description = "Tenant updated",   body = Value),
        (status = 400, description = "Bad request",      body = Value),
        (status = 404, description = "Tenant not found", body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTenantInput>,
) -> (StatusCode, Json<Value>) {
    debug!(tenant_id = %id, "PUT /auth/tenants/{id}");

    let req = UpdateTenantRequest {
        tenant_id: id,
        name:      body.name.unwrap_or_default(),
        status:    body.status.unwrap_or_default(),
    };

    let mut client = state.auth_grpc;
    match client.update_tenant(req).await {
        Ok(r) => {
            (StatusCode::OK, Json(json!({
                "tenant_id":  r.tenant_id,
                "name":       r.name,
                "status":     r.status,
                "created_at": r.created_at,
            })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}

// ── Delete tenant ─────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/auth/tenants/{id}",
    params(("id" = String, Path, description = "Tenant UUID")),
    responses(
        (status = 200, description = "Tenant deleted",   body = Value),
        (status = 404, description = "Tenant not found", body = Value),
        (status = 500, description = "Internal error",   body = Value),
    ),
    tag = "Auth"
)]
pub async fn delete_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    debug!(tenant_id = %id, "DELETE /auth/tenants/{id}");

    let req = DeleteTenantRequest { tenant_id: id };

    let mut client = state.auth_grpc;
    match client.delete_tenant(req).await {
        Ok(r) => {
            info!("DELETE /auth/tenants ← 200 success={}", r.success);
            (StatusCode::OK, Json(json!({ "success": r.success })))
        }
        Err(status) => {
            let (code, body) = grpc_to_http(status);
            (code, Json(body))
        }
    }
}
