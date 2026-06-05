// Rutas:
//   POST   /ppto/tipos-costo            → alta
//   DELETE /ppto/tipos-costo/{id}       → baja
//   PUT    /ppto/tipos-costo            → cambio
//   GET    /ppto/tipos-costo/{id}       → consulta
//   GET    /ppto/tipos-costo?activos=   → carga_tipos

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
use crate::domain::models::tipos_costo::TiposCosto;
use crate::infrastructure::db::app_state::AppState;
use crate::services::ppto::tipos_costo as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TiposCostoInput {
    pub id:          Option<i32>,
    pub nombre:      String,
    pub descripcion: String,
    pub activo:      bool,
    pub imagen:      String,
}

#[derive(Debug, Deserialize)]
pub struct TiposCostoQuery {
    pub activos: Option<bool>,
}

fn input_to_model(body: TiposCostoInput) -> TiposCosto {
    TiposCosto {
        id:          body.id,
        nombre:      body.nombre,
        descripcion: body.descripcion,
        activo:      body.activo,
        imagen:      body.imagen,
    }
}

fn tipo_json(t: &TiposCosto) -> Value {
    json!({
        "id":          t.id,
        "nombre":      t.nombre,
        "descripcion": t.descripcion,
        "activo":      t.activo,
        "imagen":      t.imagen,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/ppto/tipos-costo",
    request_body = TiposCostoInput,
    responses(
        (status = 201, description = "Tipo de costo registrado", body = Value),
        (status = 400, description = "Alta cancelada o error",   body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<TiposCostoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(nombre = %body.nombre, "POST /ppto/tipos-costo");

    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let tpo = input_to_model(body);
    let ret = svc::alta(&state.postgres, &tpo, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /ppto/tipos-costo ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /ppto/tipos-costo ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/ppto/tipos-costo/{id}",
    params(("id" = i32, Path, description = "Id del tipo de costo a eliminar")),
    responses(
        (status = 200, description = "Tipo de costo eliminado",  body = Value),
        (status = 400, description = "Baja cancelada o error",   body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /ppto/tipos-costo/{}", id);

    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        info!("DELETE /ppto/tipos-costo/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /ppto/tipos-costo/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambio ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/ppto/tipos-costo",
    request_body = TiposCostoInput,
    responses(
        (status = 200, description = "Tipo de costo actualizado",             body = Value),
        (status = 400, description = "Actualización cancelada o error",       body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<TiposCostoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /ppto/tipos-costo");

    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambio" })));
    };

    let tpo = input_to_model(body);
    let ret = svc::cambio(&state.postgres, &tpo, tenant_id).await;

    if ret.afectado > 0 {
        info!("PUT /ppto/tipos-costo ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /ppto/tipos-costo ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/tipos-costo/{id}",
    params(("id" = i32, Path, description = "Id del tipo de costo a consultar")),
    responses(
        (status = 200, description = "Tipo de costo encontrado",  body = Value),
        (status = 404, description = "Tipo de costo no encontrado", body = Value),
        (status = 500, description = "Error de base de datos",    body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /ppto/tipos-costo/{}", id);

    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(t)) => {
            info!("GET /ppto/tipos-costo/{} ← 200", id);
            (StatusCode::OK, Json(tipo_json(&t)))
        }
        Ok(None) => {
            info!("GET /ppto/tipos-costo/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Tipo de costo no encontrado" })))
        }
        Err(rc) => {
            error!("GET /ppto/tipos-costo/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Carga Tipos ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/tipos-costo",
    params(
        ("activos" = Option<bool>, Query, description = "true = solo activos, false = todos"),
    ),
    responses(
        (status = 200, description = "Lista de tipos de costo",  body = Value),
        (status = 404, description = "Sin tipos de costo",       body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn carga_tipos(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<TiposCostoQuery>,
) -> (StatusCode, Json<Value>) {
    let activos = q.activos.unwrap_or(true);
    debug!(activos, "GET /ppto/tipos-costo");

    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    match svc::carga_tipos(&state.postgres, activos, tenant_id).await {
        Ok(lista) => {
            info!("GET /ppto/tipos-costo?activos={} ← 200 {} registros", activos, lista.len());
            let items: Vec<Value> = lista.iter().map(tipo_json).collect();
            (StatusCode::OK, Json(json!({ "tipos_costo": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -60 => {
            info!("GET /ppto/tipos-costo?activos={} ← 404", activos);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/tipos-costo?activos={} ← 500 codigo={}", activos, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
