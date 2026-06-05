// Programa...: handler::catalog_g::mtto_center_costs
// Descripción: Endpoints HTTP para Centros de costo (multi-tenant)
// Origen.....: MttoCentrosCosto.aspx.cs
//
// Rutas:
//   POST   /catalog/cost-centers           → alta
//   DELETE /catalog/cost-centers/{id}      → baja
//   PUT    /catalog/cost-centers           → cambios
//   GET    /catalog/cost-centers/{id}      → consulta
//   GET    /catalog/cost-centers           → obtiene_centros  (?activos=bool, default true)
//   GET    /catalog/cost-centers/lookup    → lookup           (?q=...&limit=...)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::centros_costo::CentrosCosto;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::app_state::AppState;
use crate::services::catalog_g::mtto_center_costs as svc;

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub q:     Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CentroCostoInput {
    pub id:          Option<i32>,
    pub nombre:      String,
    pub tipo:        i32,
    pub comentarios: String,
    pub activo:      bool,
}

#[derive(Debug, Deserialize)]
pub struct FiltroActivos {
    pub activos: Option<bool>,
}

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/catalog/cost-centers",
    request_body = CentroCostoInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "Cost Centers"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CentroCostoInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /catalog/cost-centers → nombre='{}' tipo={}", body.nombre, body.tipo);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let cen = CentrosCosto {
        id:          body.id.unwrap_or(0),
        nombre:      body.nombre,
        tipo:        body.tipo,
        comentarios: body.comentarios,
        activo:      body.activo,
        tenant_id:   None, // lo fija el SP a partir del p_tenant_id
    };
    let ret = svc::alta(&state.postgres, &cen, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /catalog/cost-centers ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED,     Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /catalog/cost-centers ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/catalog/cost-centers/{id}",
    params(("id" = i32, Path, description = "Id del centro de costo a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "Cost Centers"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /catalog/cost-centers/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        info!("DELETE /catalog/cost-centers/{} ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /catalog/cost-centers/{} ← 400 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/catalog/cost-centers",
    request_body = CentroCostoInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Cost Centers"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<CentroCostoInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /catalog/cost-centers → id={:?} nombre='{}'", body.id, body.nombre);

    let Some(id) = body.id else {
        error!("PUT /catalog/cost-centers ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })),
        );
    };

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let cen = CentrosCosto {
        id,
        nombre:      body.nombre,
        tipo:        body.tipo,
        comentarios: body.comentarios,
        activo:      body.activo,
        tenant_id:   None,
    };
    let ret = svc::cambios(&state.postgres, &cen, tenant_id).await;

    if ret.afectado > 0 {
        info!("PUT /catalog/cost-centers ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /catalog/cost-centers ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/cost-centers/{id}",
    params(("id" = i32, Path, description = "Id del centro de costo a consultar")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Cost Centers"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/cost-centers/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(c)) => {
            info!("GET /catalog/cost-centers/{} ← 200 nombre='{}'", id, c.nombre);
            (StatusCode::OK, Json(json!({
                "id":          c.id,
                "nombre":      c.nombre,
                "tipo":        c.tipo,
                "comentarios": c.comentarios,
                "activo":      c.activo,
                "tenant_id":   c.tenant_id.map(|u| u.to_string()),
            })))
        }
        Ok(None) => {
            info!("GET /catalog/cost-centers/{} ← 404", id);
            (StatusCode::NOT_FOUND,             Json(json!({ "codigo": -41, "mensaje": "No existe el registro" })))
        }
        Err(ret) => {
            error!("GET /catalog/cost-centers/{} ← 500 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE CENTROS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/cost-centers",
    params(("activos" = Option<bool>, Query, description = "true = sólo activos (default), false = todos")),
    responses(
        (status = 200, description = "Lista de centros de costo", body = Value),
        (status = 404, description = "Sin registros",             body = Value),
        (status = 500, description = "Error de base de datos",    body = Value),
    ),
    tag = "Cost Centers"
)]
pub async fn obtiene_centros(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(filtro): Query<FiltroActivos>,
) -> (StatusCode, Json<Value>) {
    let activos = filtro.activos.unwrap_or(true);
    debug!("GET /catalog/cost-centers?activos={}", activos);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::obtiene_centros(&state.postgres, activos, tenant_id).await {
        Ok(lista) => {
            info!("GET /catalog/cost-centers ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|c| json!({
                "id":          c.id,
                "nombre":      c.nombre,
                "tipo":        c.tipo,
                "comentarios": c.comentarios,
                "activo":      c.activo,
                "tenant_id":   c.tenant_id.map(|u| u.to_string()),
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /catalog/cost-centers ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /catalog/cost-centers ← 404 sin registros");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// LOOKUP — autocomplete centros de costo activos
// GET /catalog/cost-centers/lookup?q=foo&limit=20
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/cost-centers/lookup",
    params(
        ("q"     = Option<String>, Query, description = "Texto a buscar (ILIKE)"),
        ("limit" = Option<i32>,    Query, description = "Máximo de resultados (default 20, máx 100)"),
    ),
    responses(
        (status = 200, description = "Lista [{id, etiqueta}]",     body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "Cost Centers"
)]
pub async fn lookup(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<LookupQuery>,
) -> (StatusCode, Json<Value>) {
    let qs    = q.q.unwrap_or_default();
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    debug!("GET /catalog/cost-centers/lookup q='{}' limit={}", qs, limit);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::lookup(&state.postgres, &qs, limit, tenant_id).await {
        Ok(items) => {
            info!("GET /catalog/cost-centers/lookup ← 200 {} items", items.len());
            let payload: Vec<LookupItem> = items;
            (StatusCode::OK, Json(json!(payload)))
        }
        Err(ret) => {
            error!("GET /catalog/cost-centers/lookup ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
