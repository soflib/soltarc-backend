// Programa...: handler::clients::project_images
// Descripción: Imágenes del proyecto para el portal de clientes
// Origen.....: Cte_Imagenes.aspx.cs
//
// Rutas:
//   GET /clients/portal/projects/{id}/images

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::project_images as svc;

#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/images",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Directorio e imágenes del proyecto", body = Value),
        (status = 404, description = "Proyecto no encontrado",             body = Value),
        (status = 500, description = "Error de base de datos",             body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn project_images(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/images", id);

    match svc::get_images(&state.postgres, id).await {
        Ok(data) => {
            info!(
                "GET /clients/portal/projects/{}/images ← 200 dir='{}'",
                id, data.directorio
            );
            (StatusCode::OK, Json(json!({
                "proyecto_id": id,
                "directorio":  data.directorio,
                "archivos":    data.archivos,
                "nota":        "Listado de archivos pendiente de configuración de almacenamiento (CC-5)",
            })))
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /clients/portal/projects/{}/images ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!(
                "GET /clients/portal/projects/{}/images ← 500 codigo={}",
                id, rc.codigo
            );
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
