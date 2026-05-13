// Programa...: handler::catalog_g::quick_access
// Descripción: Endpoints HTTP para botones de acceso rápido
// Origen.....: AccesosRapidos.aspx.cs
//
// Nota: No hay alta/baja — los registros son fijos (uno por botón del menú).
//       Solo se actualiza la función/imagen asociada a cada botón.
//
// Rutas:
//   PUT  /catalog/quick-access        → cambios
//   GET  /catalog/quick-access/{id}   → consulta

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::accesos_rapidos::AccesosRapidos;
use crate::infrastructure::db::app_state::AppState;
use crate::services::catalog_g::quick_access as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AccesosRapidosInput {
    pub id:       i32,
    pub funcion:  String,
    pub tool_tip: String,
    pub imagen:   String,
}

// ─────────────────────────────────────────────
// LISTA TODOS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/quick-access",
    responses(
        (status = 200, description = "Lista de accesos rápidos", body = Value),
        (status = 404, description = "Sin registros",            body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Quick Access"
)]
pub async fn lista_todos(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/quick-access");

    match svc::lista_todos(&state.postgres).await {
        Ok(lista) => {
            info!("GET /catalog/quick-access ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(|ar| json!({
                "id":       ar.id,
                "funcion":  ar.funcion,
                "tool_tip": ar.tool_tip,
                "imagen":   ar.imagen,
            })).collect();
            (StatusCode::OK, Json(json!({ "accesos": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /catalog/quick-access ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /catalog/quick-access ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/catalog/quick-access",
    request_body = AccesosRapidosInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Quick Access"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<AccesosRapidosInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /catalog/quick-access → id={} funcion='{}'", body.id, body.funcion);

    let ar = AccesosRapidos {
        id:       body.id,
        funcion:  body.funcion,
        tool_tip: body.tool_tip,
        imagen:   body.imagen,
    };
    let ret = svc::cambios(&state.postgres, &ar).await;

    if ret.codigo >= 0 {
        info!("PUT /catalog/quick-access ← 200 OK id={}", ar.id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /catalog/quick-access ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/quick-access/{id}",
    params(("id" = i32, Path, description = "Id del botón a consultar")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Quick Access"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/quick-access/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(ar)) => {
            info!("GET /catalog/quick-access/{} ← 200 funcion='{}'", id, ar.funcion);
            (StatusCode::OK, Json(json!({
                "id":       ar.id,
                "funcion":  ar.funcion,
                "tool_tip": ar.tool_tip,
                "imagen":   ar.imagen,
            })))
        }
        Ok(None) => {
            info!("GET /catalog/quick-access/{} ← 404", id);
            (StatusCode::NOT_FOUND,             Json(json!({ "codigo": -25, "mensaje": "No existe el registro" })))
        }
        Err(ret) => {
            error!("GET /catalog/quick-access/{} ← 500 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
