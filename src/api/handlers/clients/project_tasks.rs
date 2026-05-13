// Programa...: handler::clients::project_tasks
// Descripción: Tareas del proyecto con totales financieros para el portal de clientes
// Origen.....: Cte_ProyectosDetalleTareas.aspx.cs
//
// Rutas:
//   GET /clients/portal/projects/{id}/tasks → carga_tareas + total_egresos

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::project_tasks as svc;

#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/tasks",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Tareas del proyecto con total de egresos", body = Value),
        (status = 404, description = "Sin tareas",                               body = Value),
        (status = 500, description = "Error de base de datos",                   body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn project_tasks(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/tasks", id);

    let (tareas_res, egresos_res) = tokio::join!(
        svc::carga_tareas(&state.postgres, id),
        svc::total_egresos(&state.postgres, id),
    );

    let tareas = match tareas_res {
        Ok(t) => t,
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/tasks ← 500 carga_tareas codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/tasks ← 404 sin tareas", id);
            return (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    let total_egresos = match egresos_res {
        Ok(t) => Some(t.to_string()),
        Err(_) => None,
    };

    info!("GET /clients/portal/projects/{}/tasks ← 200 {} tareas", id, tareas.len());

    (StatusCode::OK, Json(json!({
        "proyecto_id":   id,
        "total_egresos": total_egresos,
        "tareas": tareas.iter().map(|t| json!({
            "id":           t.id,
            "proyecto":     t.proyecto,
            "tipo":         t.tipo,
            "secuencia":    t.secuencia,
            "descripcion":  t.descripcion,
            "comentarios":  t.comentarios,
            "presupuesto":  t.presupuesto.to_string(),
            "fecha_inicio": t.fecha_inicio.to_string(),
            "fecha_fin":    t.fecha_fin.to_string(),
            "fecha_termina": t.fecha_termina.to_string(),
            "estado":       t.estado,
            "nodo":         t.nodo,
        })).collect::<Vec<_>>(),
    })))
}
