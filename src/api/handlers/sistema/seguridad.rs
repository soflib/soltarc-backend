// Programa...: handler::sistema::seguridad
// Origen.....: oSeguridad.cs
//
// Rutas:
//   GET /sistema/seguridad?gpo_id=&usr_id=&usr_nivel=&usr_activo=

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::sistema::seguridad as svc;

#[derive(Debug, Deserialize)]
pub struct SeguridadQuery {
    pub gpo_id:    i32,
    pub usr_id:    i32,
    pub usr_nivel: i32,
    pub usr_activo: bool,
}

// ── Carga variables de seguridad ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/sistema/seguridad",
    params(
        ("gpo_id"     = i32,  Query, description = "Id del grupo de negocio"),
        ("usr_id"     = i32,  Query, description = "Id del usuario"),
        ("usr_nivel"  = i32,  Query, description = "Nivel del usuario"),
        ("usr_activo" = bool, Query, description = "Si el usuario está activo"),
    ),
    responses(
        (status = 200, description = "Variables de seguridad cargadas", body = Value),
        (status = 404, description = "Sin datos de seguridad",          body = Value),
        (status = 500, description = "Error de base de datos",          body = Value),
    ),
    tag = "SistemaSeg"
)]
pub async fn carga_variables(
    State(state): State<AppState>,
    Query(q): Query<SeguridadQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(gpo_id = q.gpo_id, usr_id = q.usr_id, "GET /sistema/seguridad");

    match svc::carga_variables(&state.postgres, q.gpo_id, q.usr_id, q.usr_nivel, q.usr_activo).await {
        Ok(seg) => {
            info!("GET /sistema/seguridad ← 200 OK gpo='{}'", seg.gpo_nombre);
            (StatusCode::OK, Json(json!({
                "gpo_id":      seg.gpo_id,
                "gpo_nombre":  seg.gpo_nombre,
                "gpo_descrip": seg.gpo_descrip,
                "gpo_activo":  seg.gpo_activo,
                "usr_id":      seg.usr_id,
                "usr_nivel":   seg.usr_nivel,
                "usr_activo":  seg.usr_activo,
            })))
        }
        Err(rc) if rc.codigo > -75 => {
            info!("GET /sistema/seguridad ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /sistema/seguridad ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
