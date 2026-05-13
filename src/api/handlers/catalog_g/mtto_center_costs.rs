// Programa...: handler::catalog_g::mtto_center_costs
// Descripción: Endpoints HTTP para Centros de costo
// Origen.....: MttoCentrosCosto.aspx.cs
//
// Rutas:
//   POST   /catalog/cost-centers           → alta
//   DELETE /catalog/cost-centers/{id}      → baja
//   PUT    /catalog/cost-centers           → cambios
//   GET    /catalog/cost-centers/{id}      → consulta
//   GET    /catalog/cost-centers           → obtiene_centros  (?activos=bool, default true)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::centros_costo::CentrosCosto;
use crate::infrastructure::db::app_state::AppState;
use crate::services::catalog_g::mtto_center_costs as svc;

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
    Json(body): Json<CentroCostoInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /catalog/cost-centers → nombre='{}' tipo={}", body.nombre, body.tipo);

    let cen = CentrosCosto {
        id:          body.id.unwrap_or(0),
        nombre:      body.nombre,
        tipo:        body.tipo,
        comentarios: body.comentarios,
        activo:      body.activo,
    };
    let ret = svc::alta(&state.postgres, &cen).await;

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /catalog/cost-centers/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

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
    let cen = CentrosCosto {
        id,
        nombre:      body.nombre,
        tipo:        body.tipo,
        comentarios: body.comentarios,
        activo:      body.activo,
    };
    let ret = svc::cambios(&state.postgres, &cen).await;

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/cost-centers/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(c)) => {
            info!("GET /catalog/cost-centers/{} ← 200 nombre='{}'", id, c.nombre);
            (StatusCode::OK, Json(json!({
                "id":          c.id,
                "nombre":      c.nombre,
                "tipo":        c.tipo,
                "comentarios": c.comentarios,
                "activo":      c.activo,
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
    Query(filtro): Query<FiltroActivos>,
) -> (StatusCode, Json<Value>) {
    let activos = filtro.activos.unwrap_or(true);
    debug!("GET /catalog/cost-centers?activos={}", activos);

    match svc::obtiene_centros(&state.postgres, activos).await {
        Ok(lista) => {
            info!("GET /catalog/cost-centers ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|c| json!({
                "id":          c.id,
                "nombre":      c.nombre,
                "tipo":        c.tipo,
                "comentarios": c.comentarios,
                "activo":      c.activo,
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
