// Programa...: handler::clients::weekly_plan
// Descripción: Plan semanal Gantt del portal de clientes
// Origen.....: Cte_AvancePlanSemanal.aspx.cs
//
// Rutas:
//   GET /clients/portal/projects/{id}/weekly-plan → fechas + partidas (?nivel=)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::weekly_plan as svc;

#[derive(Debug, Deserialize)]
pub struct NivelQuery {
    pub nivel: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/weekly-plan",
    params(
        ("id"    = i32,        Path,  description = "Id del proyecto"),
        ("nivel" = Option<i32>, Query, description = "Nivel de detalle (default 3)"),
    ),
    responses(
        (status = 200, description = "Plan semanal Gantt", body = Value),
        (status = 404, description = "Sin datos",          body = Value),
        (status = 500, description = "Error BD",           body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn weekly_plan(
    State(state): State<AppState>,
    axum::Extension(auth_user): axum::Extension<crate::api::middleware::roles::AuthUser>,
    Path(id): Path<i32>,
    Query(q): Query<NivelQuery>,
) -> (StatusCode, Json<Value>) {
    let nivel = q.nivel.unwrap_or(3);
    debug!("GET /clients/portal/projects/{}/weekly-plan nivel={}", id, nivel);
    if let Some(err) = super::ensure_proyecto(&state, &auth_user, id).await { return err; }

    let fechas = match svc::fechas(&state.postgres, id).await {
        Ok(f) => f,
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/weekly-plan ← 500 fechas codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/weekly-plan ← 404 sin fechas", id);
            return (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    let partidas = match svc::carga_partidas(&state.postgres, id, fechas.fecha_ini, nivel).await {
        Ok(p) => p,
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/weekly-plan ← 500 partidas codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/weekly-plan ← 404 sin partidas", id);
            return (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    info!("GET /clients/portal/projects/{}/weekly-plan ← 200 {} partidas", id, partidas.len());

    (StatusCode::OK, Json(json!({
        "fechas": {
            "fecha_ini":   fechas.fecha_ini,
            "fecha_fin":   fechas.fecha_fin,
            "num_semanas": fechas.num_semanas,
        },
        "partidas": partidas.iter().map(|p| json!({
            "nodo":        p.nodo,
            "nivel":       p.nivel,
            "descripcion": p.descripcion,
            "fecha_inicio": p.fecha_inicio,
            "fecha_fin":   p.fecha_fin,
            "cuando_ini":  p.cuando_ini,
            "cuando_fin":  p.cuando_fin,
            "estado":      p.estado,
        })).collect::<Vec<_>>(),
    })))
}
