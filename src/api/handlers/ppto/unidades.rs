// Rutas:
//   POST   /ppto/unidades        → alta
//   DELETE /ppto/unidades/{id}   → baja
//   PUT    /ppto/unidades        → cambio
//   GET    /ppto/unidades/{id}   → consulta
//   GET    /ppto/unidades        → obtiene_unidades

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::unidades::Unidades;
use crate::infrastructure::db::app_state::AppState;
use crate::services::ppto::unidades as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UnidadesInput {
    pub id:           Option<i32>,
    pub tipo:         i32,
    pub descripcion:  String,
    pub nombre_corto: String,
    pub activa:       bool,
}

fn input_to_model(body: UnidadesInput) -> Unidades {
    Unidades {
        id:           body.id,
        tipo:         body.tipo,
        descripcion:  body.descripcion,
        nombre_corto: body.nombre_corto,
        activa:       body.activa,
    }
}

fn unidad_json(u: &Unidades) -> Value {
    json!({
        "id":           u.id,
        "tipo":         u.tipo,
        "descripcion":  u.descripcion,
        "nombre_corto": u.nombre_corto,
        "activa":       u.activa,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/ppto/unidades",
    request_body = UnidadesInput,
    responses(
        (status = 201, description = "Unidad registrada",       body = Value),
        (status = 400, description = "Alta cancelada o error",  body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<UnidadesInput>,
) -> (StatusCode, Json<Value>) {
    debug!(nombre_corto = %body.nombre_corto, "POST /ppto/unidades");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let uni = input_to_model(body);
    let ret = svc::alta(&state.postgres, &uni, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /ppto/unidades ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /ppto/unidades ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/ppto/unidades/{id}",
    params(("id" = i32, Path, description = "Id de la unidad a eliminar")),
    responses(
        (status = 200, description = "Unidad eliminada",        body = Value),
        (status = 400, description = "Baja cancelada o error",  body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /ppto/unidades/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        info!("DELETE /ppto/unidades/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /ppto/unidades/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambio ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/ppto/unidades",
    request_body = UnidadesInput,
    responses(
        (status = 200, description = "Unidad actualizada",                   body = Value),
        (status = 400, description = "Actualización cancelada o error",      body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<UnidadesInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /ppto/unidades");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambio" })));
    };

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let uni = input_to_model(body);
    let ret = svc::cambio(&state.postgres, &uni, tenant_id).await;

    if ret.afectado > 0 {
        info!("PUT /ppto/unidades ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /ppto/unidades ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/unidades/{id}",
    params(("id" = i32, Path, description = "Id de la unidad a consultar")),
    responses(
        (status = 200, description = "Unidad encontrada",       body = Value),
        (status = 404, description = "Unidad no encontrada",    body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /ppto/unidades/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(u)) => {
            info!("GET /ppto/unidades/{} ← 200", id);
            (StatusCode::OK, Json(unidad_json(&u)))
        }
        Ok(None) => {
            info!("GET /ppto/unidades/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Unidad no encontrada" })))
        }
        Err(rc) => {
            error!("GET /ppto/unidades/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Carga Árbol ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/unidades/arbol",
    responses(
        (status = 200, description = "Árbol de unidades",        body = Value),
        (status = 404, description = "Sin unidades",             body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn carga_arbol(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /ppto/unidades/arbol");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::carga_arbol(&state.postgres, tenant_id).await {
        Ok(lista) => {
            info!("GET /ppto/unidades/arbol ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(unidad_json).collect();
            (StatusCode::OK, Json(json!({ "unidades": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -20 => {
            info!("GET /ppto/unidades/arbol ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/unidades/arbol ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Obtiene Unidades ──────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/unidades",
    responses(
        (status = 200, description = "Lista de unidades",       body = Value),
        (status = 404, description = "Sin unidades",            body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn obtiene_unidades(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /ppto/unidades");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::obtiene_unidades(&state.postgres, tenant_id).await {
        Ok(lista) => {
            info!("GET /ppto/unidades ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(unidad_json).collect();
            (StatusCode::OK, Json(json!({ "unidades": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -20 => {
            info!("GET /ppto/unidades ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/unidades ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
