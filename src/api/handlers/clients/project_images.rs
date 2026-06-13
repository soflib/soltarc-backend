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
    axum::Extension(auth_user): axum::Extension<crate::api::middleware::roles::AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/images", id);
    if let Some(err) = super::ensure_proyecto(&state, &auth_user, id).await { return err; }

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::get_images(&state.postgres, id).await {
        Ok(data) => {
            // Archivos reales desde Contabo (metadata en cpa_tenant_archivos +
            // URL presignada de 1h). Si el storage no está configurado, lista
            // vacía con nota — el portal degrada con su aviso.
            let (archivos, nota) = match &state.storage {
                Some(storage) => {
                    let lista = crate::dal::archivos::lista_proyecto(&state.postgres, tenant_id, id)
                        .await
                        .unwrap_or_default();
                    let mut out = Vec::with_capacity(lista.len());
                    for a in &lista {
                        let url = storage.presigned_get(&a.s3_key, 3600).await.unwrap_or_default();
                        out.push(json!({
                            "id": a.id, "nombre": a.nombre, "mime": a.mime,
                            "bytes": a.bytes, "url": url, "created_at": a.created_at.to_rfc3339(),
                        }));
                    }
                    (out, Value::Null)
                }
                None => (vec![], json!("Almacenamiento no configurado en el servidor.")),
            };

            info!(
                "GET /clients/portal/projects/{}/images ← 200 dir='{}' archivos={}",
                id, data.directorio, archivos.len()
            );
            (StatusCode::OK, Json(json!({
                "proyecto_id": id,
                "directorio":  data.directorio,
                "archivos":    archivos,
                "nota":        nota,
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
