// Programa...: handler::clients::work_progress
// Descripción: Avance de obra del portal de clientes
// Origen.....: Cte_AvanceDeObra.aspx.cs
//
// Rutas:
//   GET /clients/portal/projects/{id}/work-progress → datos combinados

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::work_progress as svc;

#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/work-progress",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Avance de obra: proyecto, ingresos, egresos", body = Value),
        (status = 404, description = "Proyecto no encontrado",                      body = Value),
        (status = 500, description = "Error de base de datos",                      body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn work_progress(
    State(state): State<AppState>,
    Extension(auth_user): Extension<crate::api::middleware::roles::AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/work-progress", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let proyecto = match svc::consulta_proyecto(&state.postgres, id, tenant_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            info!("GET /clients/portal/projects/{}/work-progress ← 404", id);
            return (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Proyecto no encontrado" })));
        }
        Err(ret) => {
            error!("GET /clients/portal/projects/{}/work-progress ← 500 consulta_proyecto codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    let nombre_cliente = svc::nombre_cliente(&state.postgres, proyecto.cliente, tenant_id).await;

    let (ingresos, egresos) = tokio::join!(
        svc::ingresos(&state.postgres, id),
        svc::egresos(&state.postgres, id),
    );

    let ingresos = match ingresos {
        Ok(v) => v,
        Err(ret) => {
            error!("GET /clients/portal/projects/{}/work-progress ← 500 ingresos codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    let egresos = match egresos {
        Ok(v) => v,
        Err(ret) => {
            error!("GET /clients/portal/projects/{}/work-progress ← 500 egresos codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    info!(
        "GET /clients/portal/projects/{}/work-progress ← 200 ing={} egr={}",
        id, ingresos.len(), egresos.len()
    );

    (StatusCode::OK, Json(json!({
        "proyecto": {
            "id":          proyecto.id,
            "nombre":      proyecto.nombre,
            "descripcion": proyecto.descripcion,
            "cliente":     proyecto.cliente,
            "nombre_cliente": nombre_cliente.mensaje,
            "presupuesto": proyecto.presupuesto.to_string(),
            "fecha_ini":   proyecto.fecha_ini.to_string(),
            "fecha_fin":   proyecto.fecha_fin.to_string(),
            "estado":      proyecto.estado,
        },
        "ingresos": ingresos.iter().map(|r| json!({
            "fecha":      r.fecha,
            "concepto":   r.concepto,
            "referencia": r.referencia,
            "proyecto":   r.proyecto,
            "monto":      r.monto.as_ref().map(|d| d.to_string()),
            "usuario":    r.usuario,
        })).collect::<Vec<_>>(),
        "egresos": egresos.iter().map(|r| json!({
            "fecha":      r.fecha,
            "concepto":   r.concepto,
            "referencia": r.referencia,
            "proyecto":   r.proyecto,
            "monto":      r.monto.as_ref().map(|d| d.to_string()),
            "usuario":    r.usuario,
        })).collect::<Vec<_>>(),
    })))
}
