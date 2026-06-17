// Programa...: handler::sistema::logo
// Descripción: Logo de la empresa POR TENANT en Contabo (object storage), para
//              mostrarlo/editarlo en Configuración y usarlo en los reportes PDF.
//
// Rutas (todas bajo require_arquitecto):
//   GET    /sistema/logo  → URL presignada del logo del tenant (o null)
//   POST   /sistema/logo  → sube/reemplaza el logo (multipart "archivo", imagen ≤2MB)
//   DELETE /sistema/logo  → elimina el logo
//
// La key es fija por tenant ({tenant}/config/logo), así que subir = sobrescribe.
// NO se guarda nada en la config (que es global, no por tenant): el logo queda
// determinado únicamente por el tenant_id. Si Contabo no está configurado → 503.

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{error, info};

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::storage::contabo::{keys, ContaboStorage};

const MAX_LOGO_BYTES: usize = 2 * 1024 * 1024; // 2MB
const PRESIGN_LOGO_SECS: u32 = 3600; // 1h

fn storage_or_503(state: &AppState) -> Result<Arc<ContaboStorage>, (StatusCode, Json<Value>)> {
    state.storage.clone().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "mensaje": "Almacenamiento no configurado en el servidor." })),
    ))
}

/// GET /sistema/logo → `{ "url": <presignada> }` o `{ "url": null }` si no hay.
pub async fn obtener(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };
    let key = keys::tenant_logo(&tenant_id);

    match storage.list_keys(&key).await {
        Ok(found) if found.iter().any(|k| k == &key) => {
            match storage.presigned_get(&key, PRESIGN_LOGO_SECS).await {
                Ok(url) => (StatusCode::OK, Json(json!({ "url": url }))),
                Err(e) => {
                    error!("GET /sistema/logo ← presign: {e}");
                    (StatusCode::BAD_GATEWAY, Json(json!({ "mensaje": "No se pudo generar el enlace del logo." })))
                }
            }
        }
        Ok(_) => (StatusCode::OK, Json(json!({ "url": Value::Null }))),
        Err(e) => {
            error!("GET /sistema/logo ← list: {e}");
            (StatusCode::BAD_GATEWAY, Json(json!({ "mensaje": "No se pudo consultar el logo." })))
        }
    }
}

/// POST /sistema/logo (multipart "archivo") → sube/reemplaza el logo del tenant.
pub async fn subir(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let mut data: Option<Vec<u8>> = None;
    let mut mime = String::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let is_file = field.file_name().is_some();
        let fname = field.name().unwrap_or_default().to_string();
        let ctype = field.content_type().unwrap_or("").to_string();
        if is_file || fname == "archivo" {
            mime = ctype;
            match field.bytes().await {
                Ok(b) => data = Some(b.to_vec()),
                Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Imagen inválida o demasiado grande." }))),
            }
            break;
        }
    }

    let data = match data {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "No se recibió ninguna imagen." }))),
    };
    if !mime.starts_with("image/") {
        return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "El logo debe ser una imagen (PNG, JPG o SVG)." })));
    }
    if data.len() > MAX_LOGO_BYTES {
        return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "El logo no debe exceder 2MB." })));
    }

    let key = keys::tenant_logo(&tenant_id);
    if let Err(e) = storage.upload(&key, &data, &mime).await {
        error!("POST /sistema/logo ← upload: {e}");
        return (StatusCode::BAD_GATEWAY, Json(json!({ "mensaje": "No se pudo subir el logo al almacenamiento." })));
    }
    let url = storage.presigned_get(&key, PRESIGN_LOGO_SECS).await.unwrap_or_default();
    info!("POST /sistema/logo ← 200 tenant={}", tenant_id);
    (StatusCode::OK, Json(json!({ "url": url, "mensaje": "Logo actualizado." })))
}

/// DELETE /sistema/logo → elimina el logo del tenant.
pub async fn borrar(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };
    let key = keys::tenant_logo(&tenant_id);

    match storage.delete_object(&key).await {
        Ok(_) => (StatusCode::OK, Json(json!({ "mensaje": "Logo eliminado." }))),
        Err(e) => {
            error!("DELETE /sistema/logo ← {e}");
            (StatusCode::BAD_GATEWAY, Json(json!({ "mensaje": "No se pudo eliminar el logo." })))
        }
    }
}
