// Programa...: handler::clients::dashboard
// Descripción: Dashboard del portal de clientes
// Origen.....: Cte_Inicial.aspx.cs
//
// Rutas:
//   GET /clients/portal/dashboard           → llena_det_proyectos (?grupo=&usuario=&nivel=)
//   GET /clients/portal/projects/{id}/ppto  → total_ppto

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::dashboard as svc;

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub grupo:   i32,
    pub usuario: i32,
    pub nivel:   i32,
}

// ─────────────────────────────────────────────
// LLENA DET PROYECTOS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/clients/portal/dashboard",
    params(
        ("grupo"   = i32, Query, description = "Id de grupo"),
        ("usuario" = i32, Query, description = "Id de usuario"),
        ("nivel"   = i32, Query, description = "Nivel de acceso"),
    ),
    responses(
        (status = 200, description = "Lista de proyectos del cliente", body = Value),
        (status = 404, description = "Sin proyectos",                  body = Value),
        (status = 500, description = "Error de base de datos",         body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn llena_det_proyectos(
    State(state): State<AppState>,
    Query(q): Query<DashboardQuery>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/dashboard grupo={} usuario={} nivel={}", q.grupo, q.usuario, q.nivel);

    match svc::llena_det_proyectos(&state.postgres, q.grupo, q.usuario, q.nivel).await {
        Ok(lista) => {
            info!("GET /clients/portal/dashboard ← 200 {} proyectos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|p| json!({
                "proyecto_id":    p.proyecto_id,
                "nombre":         p.nombre,
                "presupuesto":    p.presupuesto.as_ref().map(|d| d.to_string()),
                "total_ingresos": p.total_ingresos.as_ref().map(|d| d.to_string()),
                "total_egresos":  p.total_egresos.as_ref().map(|d| d.to_string()),
                "saldo":          p.saldo.as_ref().map(|d| d.to_string()),
                "estado":         p.estado,
                "cliente":        p.cliente,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/dashboard ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/dashboard ← 404 sin proyectos");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// TOTAL PPTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/ppto",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Total presupuesto", body = Value),
        (status = 404, description = "Sin datos",         body = Value),
        (status = 500, description = "Error BD",          body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn total_ppto(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/ppto", id);

    match svc::total_ppto(&state.postgres, id).await {
        Ok(total) => {
            info!("GET /clients/portal/projects/{}/ppto ← 200 total={}", id, total);
            (StatusCode::OK, Json(json!({ "proyecto_id": id, "total_ppto": total.to_string() })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/ppto ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/ppto ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
