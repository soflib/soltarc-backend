// Programa...: handler::sistema::gn_grupos
// Origen.....: oGNGrupos.cs
//
// Rutas:
//   POST   /sistema/grupos              → alta
//   DELETE /sistema/grupos/{id}         → baja
//   PUT    /sistema/grupos              → cambios
//   GET    /sistema/grupos/{id}         → consulta
//   GET    /sistema/grupos?activos=     → obtiene_todo

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::domain::models::gn_grupos::GnGrupos;
use crate::infrastructure::db::app_state::AppState;
use crate::services::sistema::gn_grupos as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GnGruposInput {
    pub id:          Option<i32>,
    pub nombre:      String,
    pub descripcion: String,
    pub activo:      bool,
}

#[derive(Debug, Deserialize)]
pub struct GruposQuery {
    pub activos: Option<bool>,
}

fn to_model(body: GnGruposInput) -> GnGrupos {
    GnGrupos {
        id:          body.id.unwrap_or(0),
        nombre:      body.nombre,
        descripcion: body.descripcion,
        activo:      body.activo,
    }
}

fn grupo_json(g: &GnGrupos) -> Value {
    json!({
        "id":          g.id,
        "nombre":      g.nombre,
        "descripcion": g.descripcion,
        "activo":      g.activo,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/sistema/grupos",
    request_body = GnGruposInput,
    responses(
        (status = 201, description = "Grupo registrado",          body = Value),
        (status = 400, description = "Alta cancelada o error",    body = Value),
    ),
    tag = "SistemaGrupos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<GnGruposInput>,
) -> (StatusCode, Json<Value>) {
    debug!(nombre = %body.nombre, "POST /sistema/grupos");

    let gpo = to_model(body);
    let ret = svc::alta(&state.postgres, &gpo).await;

    if ret.afectado > 0 {
        info!("POST /sistema/grupos ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /sistema/grupos ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/sistema/grupos/{id}",
    params(("id" = i32, Path, description = "Id del grupo a eliminar")),
    responses(
        (status = 200, description = "Grupo eliminado",        body = Value),
        (status = 400, description = "Baja cancelada o error", body = Value),
    ),
    tag = "SistemaGrupos"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /sistema/grupos/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /sistema/grupos/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambios ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/sistema/grupos",
    request_body = GnGruposInput,
    responses(
        (status = 200, description = "Grupo actualizado",                    body = Value),
        (status = 400, description = "Actualización cancelada o error",      body = Value),
    ),
    tag = "SistemaGrupos"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<GnGruposInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /sistema/grupos");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let gpo = to_model(body);
    let ret = svc::cambios(&state.postgres, &gpo).await;

    if ret.afectado > 0 {
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /sistema/grupos ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/sistema/grupos/{id}",
    params(("id" = i32, Path, description = "Id del grupo a consultar")),
    responses(
        (status = 200, description = "Grupo encontrado",      body = Value),
        (status = 404, description = "Grupo no encontrado",   body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "SistemaGrupos"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /sistema/grupos/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(g)) => (StatusCode::OK, Json(grupo_json(&g))),
        Ok(None)    => (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Grupo no encontrado" }))),
        Err(rc)     => {
            error!("GET /sistema/grupos/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Obtiene todo ──────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/sistema/grupos",
    params(
        ("activos" = Option<bool>, Query, description = "true = solo activos, false = todos"),
    ),
    responses(
        (status = 200, description = "Lista de grupos (vacía si no hay registros)", body = Value),
        (status = 500, description = "Error de base de datos",                      body = Value),
    ),
    tag = "SistemaGrupos"
)]
pub async fn obtiene_todo(
    State(state): State<AppState>,
    Query(q): Query<GruposQuery>,
) -> (StatusCode, Json<Value>) {
    let activos = q.activos.unwrap_or(true);
    debug!(activos, "GET /sistema/grupos");

    match svc::obtiene_todo(&state.postgres, activos).await {
        Ok(lista) => {
            info!("GET /sistema/grupos?activos={} ← 200 {} registros", activos, lista.len());
            let items: Vec<Value> = lista.iter().map(grupo_json).collect();
            (StatusCode::OK, Json(json!({ "grupos": items, "total": items.len() })))
        }
        // DAL returns codigo -21 ("No hay entradas") when the table is empty.
        // Treat that as an empty list (200), not a missing resource (404), so callers
        // that combine multiple list calls don't blow up on first-time setup.
        Err(rc) if rc.codigo == -21 => {
            info!("GET /sistema/grupos?activos={} ← 200 [] (sin registros)", activos);
            (StatusCode::OK, Json(json!({ "grupos": [], "total": 0 })))
        }
        Err(rc) => {
            error!("GET /sistema/grupos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
