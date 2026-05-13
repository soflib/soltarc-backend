// Rutas:
//   POST   /ppto/costos-estimados              → alta
//   DELETE /ppto/costos-estimados/{id}         → baja
//   PUT    /ppto/costos-estimados              → cambios
//   GET    /ppto/costos-estimados/{id}         → consulta
//   GET    /ppto/costos-estimados?activos=     → carga_arbol (obtiene_activos)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::costos_estimados::CostosEstimados;
use crate::infrastructure::db::app_state::AppState;
use crate::services::ppto::costos_estimados as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CostosEstimadosInput {
    pub id:          Option<i32>,
    pub tipo:        i32,
    pub nombre:      String,
    pub descripcion: String,
    pub unidad:      i32,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha:       String,
    pub importe:     f64,
    pub activo:      bool,
}

#[derive(Debug, Deserialize)]
pub struct CostosEstimadosQuery {
    pub activos: Option<bool>,
}

fn parse_input(body: CostosEstimadosInput) -> Result<CostosEstimados, String> {
    let fecha = NaiveDateTime::parse_from_str(&body.fecha, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("fecha inválida: {e}"))?;
    let importe = Decimal::try_from(body.importe)
        .map_err(|e| format!("importe inválido: {e}"))?;

    Ok(CostosEstimados {
        id:          body.id.unwrap_or(0),
        tipo:        body.tipo,
        nombre:      body.nombre,
        descripcion: body.descripcion,
        unidad:      body.unidad,
        fecha,
        importe,
        activo:      body.activo,
    })
}

fn costo_json(c: &CostosEstimados) -> Value {
    json!({
        "id":          c.id,
        "tipo":        c.tipo,
        "nombre":      c.nombre,
        "descripcion": c.descripcion,
        "unidad":      c.unidad,
        "fecha":       c.fecha.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "importe":     c.importe.to_string(),
        "activo":      c.activo,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/ppto/costos-estimados",
    request_body = CostosEstimadosInput,
    responses(
        (status = 201, description = "Costo estimado registrado", body = Value),
        (status = 400, description = "Alta cancelada o error",    body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<CostosEstimadosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(nombre = %body.nombre, "POST /ppto/costos-estimados");

    let cos = match parse_input(body) {
        Ok(c)    => c,
        Err(msg) => {
            error!("POST /ppto/costos-estimados ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &cos).await;

    if ret.afectado > 0 {
        info!("POST /ppto/costos-estimados ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /ppto/costos-estimados ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/ppto/costos-estimados/{id}",
    params(("id" = i32, Path, description = "Id del costo estimado a eliminar")),
    responses(
        (status = 200, description = "Costo estimado eliminado", body = Value),
        (status = 400, description = "Baja cancelada o error",   body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /ppto/costos-estimados/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /ppto/costos-estimados/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /ppto/costos-estimados/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambios ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/ppto/costos-estimados",
    request_body = CostosEstimadosInput,
    responses(
        (status = 200, description = "Costo estimado actualizado",            body = Value),
        (status = 400, description = "Actualización cancelada o error",       body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<CostosEstimadosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /ppto/costos-estimados");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let cos = match parse_input(body) {
        Ok(c)    => c,
        Err(msg) => {
            error!("PUT /ppto/costos-estimados ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambios(&state.postgres, &cos).await;

    if ret.afectado > 0 {
        info!("PUT /ppto/costos-estimados ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /ppto/costos-estimados ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/costos-estimados/{id}",
    params(("id" = i32, Path, description = "Id del costo estimado a consultar")),
    responses(
        (status = 200, description = "Costo estimado encontrado",  body = Value),
        (status = 404, description = "Costo estimado no encontrado", body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /ppto/costos-estimados/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(c)) => {
            info!("GET /ppto/costos-estimados/{} ← 200", id);
            (StatusCode::OK, Json(costo_json(&c)))
        }
        Ok(None) => {
            info!("GET /ppto/costos-estimados/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Costo estimado no encontrado" })))
        }
        Err(rc) => {
            error!("GET /ppto/costos-estimados/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Carga Arbol (obtiene_activos) ─────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/costos-estimados",
    params(
        ("activos" = Option<bool>, Query, description = "true = solo activos, false = todos"),
    ),
    responses(
        (status = 200, description = "Lista de costos estimados",  body = Value),
        (status = 404, description = "Sin costos estimados",       body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "PptoCatalogos"
)]
pub async fn carga_arbol(
    State(state): State<AppState>,
    Query(q): Query<CostosEstimadosQuery>,
) -> (StatusCode, Json<Value>) {
    let activos = q.activos.unwrap_or(true);
    debug!(activos, "GET /ppto/costos-estimados");

    match svc::carga_arbol(&state.postgres, activos).await {
        Ok(lista) => {
            info!("GET /ppto/costos-estimados?activos={} ← 200 {} registros", activos, lista.len());
            let items: Vec<Value> = lista.iter().map(costo_json).collect();
            (StatusCode::OK, Json(json!({ "costos_estimados": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -75 => {
            info!("GET /ppto/costos-estimados?activos={} ← 404", activos);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/costos-estimados?activos={} ← 500 codigo={}", activos, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
