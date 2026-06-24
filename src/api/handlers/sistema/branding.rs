// Programa...: handler::sistema::branding
// Descripción: Marca POR TENANT (nombre + logo) para el sidebar del dashboard.
//
// Rutas:
//   GET /sistema/branding  → { nombre, logo_url }  (cualquier usuario autenticado)
//   PUT /sistema/nombre    → guarda el nombre de marca (solo Admin/Arquitecto)
//
// El nombre se guarda como un objeto de texto plano en object storage, en la key
// fija {tenant}/config/brand_name (mismo patrón que el logo en {tenant}/config/logo).
// NO se usa la config (que es global, no por tenant). Si no hay nombre/logo, el
// frontend usa los valores por defecto ("SoltArc" + iniciales).
//
// El GET es de solo lectura y lo necesitan TODOS los roles (cualquier usuario ve el
// sidebar), por eso vive en el grupo "any authenticated user" y NO bajo
// require_arquitecto. El PUT sí va bajo require_arquitecto.

use axum::{
    extract::State,
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{error, info};

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::storage::contabo::keys;

const PRESIGN_LOGO_SECS: u32 = 3600; // 1h
const MAX_NAME_CHARS: usize = 40;

#[derive(Deserialize)]
pub struct NombrePayload {
    pub nombre: Option<String>,
}

/// GET /sistema/branding → `{ "nombre": <string|null>, "logo_url": <string|null> }`.
/// Cualquier usuario autenticado. Si el storage no está configurado o no hay datos,
/// devuelve nulos (200) para que el sidebar use los valores por defecto.
pub async fn obtener(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };
    let storage = match state.storage.clone() {
        Some(s) => s,
        None => return (StatusCode::OK, Json(json!({ "nombre": Value::Null, "logo_url": Value::Null }))),
    };

    // Nombre de marca (objeto de texto plano).
    let name_key = keys::tenant_brand_name(&tenant_id);
    let nombre = match storage.list_keys(&name_key).await {
        Ok(found) if found.iter().any(|k| k == &name_key) => match storage.download(&name_key).await {
            Ok(bytes) => {
                let s = String::from_utf8_lossy(&bytes).trim().to_string();
                if s.is_empty() { Value::Null } else { Value::String(s) }
            }
            Err(e) => {
                error!("GET /sistema/branding ← download name: {e}");
                Value::Null
            }
        },
        _ => Value::Null,
    };

    // Logo (URL presignada), mismo criterio que GET /sistema/logo.
    let logo_key = keys::tenant_logo(&tenant_id);
    let logo_url = match storage.list_keys(&logo_key).await {
        Ok(found) if found.iter().any(|k| k == &logo_key) => storage
            .presigned_get(&logo_key, PRESIGN_LOGO_SECS)
            .await
            .map(Value::String)
            .unwrap_or(Value::Null),
        _ => Value::Null,
    };

    (StatusCode::OK, Json(json!({ "nombre": nombre, "logo_url": logo_url })))
}

/// PUT /sistema/nombre (JSON `{ "nombre": "..." }`) → guarda/borra el nombre de marca.
/// Vacío = restablecer al valor por defecto (borra el objeto). Solo Admin/Arquitecto.
pub async fn guardar_nombre(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(payload): Json<NombrePayload>,
) -> (StatusCode, Json<Value>) {
    let storage = match state.storage.clone() {
        Some(s) => s,
        None => return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "mensaje": "Almacenamiento no configurado en el servidor." })),
        ),
    };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };
    let key = keys::tenant_brand_name(&tenant_id);

    let nombre = payload.nombre.unwrap_or_default();
    let nombre = nombre.trim();
    if nombre.chars().count() > MAX_NAME_CHARS {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "mensaje": "El nombre no debe exceder 40 caracteres." })),
        );
    }

    if nombre.is_empty() {
        // Vacío = restablecer al valor por defecto.
        let _ = storage.delete_object(&key).await;
        info!("PUT /sistema/nombre ← reset tenant={}", tenant_id);
        return (StatusCode::OK, Json(json!({ "nombre": Value::Null, "mensaje": "Nombre restablecido." })));
    }

    if let Err(e) = storage.upload(&key, nombre.as_bytes(), "text/plain; charset=utf-8").await {
        error!("PUT /sistema/nombre ← upload: {e}");
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "mensaje": "No se pudo guardar el nombre." })),
        );
    }
    info!("PUT /sistema/nombre ← 200 tenant={}", tenant_id);
    (StatusCode::OK, Json(json!({ "nombre": nombre, "mensaje": "Nombre actualizado." })))
}
