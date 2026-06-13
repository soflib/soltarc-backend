// Programa...: handler::clients::project_tree
// Descripción: Árbol de tareas del proyecto para el portal de clientes
// Origen.....: Cte_ArbolProyecto.aspx.cs
//
// Rutas:
//   GET /clients/portal/projects/{id}/tree → arbol_tareas

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::project_tree as svc;

#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/tree",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Árbol de partidas del proyecto", body = Value),
        (status = 404, description = "Sin datos",                      body = Value),
        (status = 500, description = "Error de base de datos",         body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn arbol_tareas(
    State(state): State<AppState>,
    axum::Extension(auth_user): axum::Extension<crate::api::middleware::roles::AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/tree", id);
    if let Some(err) = super::ensure_proyecto(&state, &auth_user, id).await { return err; }

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let (lista_res, estados) = tokio::join!(
        svc::arbol_tareas(&state.postgres, id),
        crate::services::clients::catalogo_map(&state.postgres, crate::services::clients::CAT_ESTADO_PARTIDAS, tenant_id),
    );

    match lista_res {
        Ok(lista) => {
            info!("GET /clients/portal/projects/{}/tree ← 200 {} nodos", id, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|p| json!({
                "nodo":          p.nodo,
                "nivel":         p.nivel,
                "descripcion":   p.descripcion,
                "estado":        p.estado,
                "estado_nombre": p.estado.and_then(|e| estados.get(&e)),
                "proyecto":      p.proyecto,
                "importe":       p.importe.as_ref().map(|d| d.to_string()),
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/tree ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/tree ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
