// Programa...: handler::clients::expense_detail
// Descripción: Detalle de gastos por partida para el portal de clientes
// Origen.....: Cte_DetalleProyXrefGasto.aspx.cs
//
// Rutas:
//   GET /clients/portal/projects/{id}/expense-detail → consulta_partidas_xref

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::expense_detail as svc;

#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/expense-detail",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Detalle de partidas con gastos", body = Value),
        (status = 404, description = "Sin datos",                      body = Value),
        (status = 500, description = "Error de base de datos",         body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn expense_detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/expense-detail", id);

    match svc::consulta_partidas_xref(&state.postgres, id).await {
        Ok(lista) => {
            info!("GET /clients/portal/projects/{}/expense-detail ← 200 {} partidas", id, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|p| json!({
                "id":           p.id,
                "proyecto":     p.proyecto,
                "tipo":         p.tipo,
                "secuencia":    p.secuencia,
                "descripcion":  p.descripcion,
                "comentarios":  p.comentarios,
                "presupuesto":  p.presupuesto.to_string(),
                "fecha_inicio": p.fecha_inicio.to_string(),
                "fecha_fin":    p.fecha_fin.to_string(),
                "fecha_termina": p.fecha_termina.to_string(),
                "estado":       p.estado,
                "nodo":         p.nodo,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/expense-detail ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/expense-detail ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
