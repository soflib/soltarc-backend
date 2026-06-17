// Programa...: handler::operaciones::archivos
// Descripción: Archivos del tenant (fotos/videos/PDF por proyecto) en Contabo
//              + capturas del formulario de Soporte (prefijo support/).
//
// Rutas:
//   POST   /operaciones/proyectos/{id}/archivos → subir   (multipart, ≤200MB c/u)
//   GET    /operaciones/proyectos/{id}/archivos → listar  (metadata + URL presignada 1h)
//   DELETE /operaciones/archivos/{id}           → borrar  (bucket + metadata)
//   GET    /operaciones/storage/uso             → uso     (bytes usados vs cuota 25GB)
//   POST   /sistema/soporte                     → soporte (imágenes ≤5MB → support/)
//
// La cuota se valida en sp_cpa_archivo_add (−30 = llena → 409). Si Contabo no
// está configurado (AppState.storage = None) todos responden 503.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::api::middleware::roles::AuthUser;
use crate::dal::archivos as dal;
use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::storage::contabo::{keys, ContaboStorage};

const MAX_FILE_BYTES: usize = 200 * 1024 * 1024;      // 200MB por archivo de proyecto
const MAX_SUPPORT_IMG_BYTES: usize = 5 * 1024 * 1024; // 5MB por captura de soporte
const MAX_SUPPORT_IMGS: usize = 5;
const PRESIGN_GALLERY_SECS: u32 = 3600;               // 1h para galerías
const PRESIGN_SUPPORT_SECS: u32 = 604_800;            // 7 días para links del correo de soporte

fn storage_or_503(state: &AppState) -> Result<Arc<ContaboStorage>, (StatusCode, Json<Value>)> {
    state.storage.clone().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "mensaje": "Almacenamiento no configurado en el servidor." })),
    ))
}

fn mime_permitido(mime: &str) -> bool {
    mime.starts_with("image/") || mime.starts_with("video/") || mime == "application/pdf"
}

// ─────────────────────────────────────────────
// SUBIR — multipart con uno o más archivos
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/proyectos/{id}/archivos",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 201, description = "Archivos subidos",                       body = Value),
        (status = 400, description = "Tipo o tamaño de archivo no permitido",  body = Value),
        (status = 409, description = "Cuota de almacenamiento llena (25GB)",   body = Value),
        (status = 503, description = "Almacenamiento no configurado",          body = Value),
    ),
    tag = "Archivos"
)]
pub async fn subir(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    debug!("POST /operaciones/proyectos/{}/archivos", id);
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let mut subidos: Vec<Value> = Vec::new();
    // Área de obra; el campo de texto "area" debe llegar ANTES de los archivos.
    let mut area = keys::AREA_DEFAULT.to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        let Some(fname) = field.file_name().map(str::to_string) else {
            // campo de texto (p. ej. "area"): clasifica los archivos que siguen
            if field.name() == Some("area") {
                area = keys::normalize_area(&field.text().await.unwrap_or_default());
            }
            continue;
        };
        let mime = field.content_type().unwrap_or("application/octet-stream").to_string();

        if !mime_permitido(&mime) {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "mensaje": format!("Tipo de archivo no permitido: {mime}. Solo imágenes, videos y PDF.")
            })));
        }
        let data = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!("POST archivos ← error leyendo multipart: {}", e);
                return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Archivo demasiado grande o inválido." })));
            }
        };
        if data.len() > MAX_FILE_BYTES {
            return (StatusCode::BAD_REQUEST, Json(json!({
                "mensaje": format!("'{fname}' excede el máximo de 200MB por archivo.")
            })));
        }

        // 1) metadata + cuota (sp_cpa_archivo_add valida los 25GB)
        let key = keys::tenant_proyecto_file(&tenant_id, id, &area, &fname);
        let archivo_id = match dal::alta(
            &state.postgres, tenant_id, Some(id), &key, &fname, &mime,
            data.len() as i64, &auth_user.username,
        ).await {
            Ok(aid) => aid,
            Err(rc) if rc.codigo == -30 => {
                info!("POST archivos ← 409 cuota llena tenant={}", tenant_id);
                return (StatusCode::CONFLICT, Json(json!({
                    "codigo": -30,
                    "mensaje": "Espacio de almacenamiento lleno (25GB). Libera espacio o contáctanos.",
                    "subidos": subidos,
                })));
            }
            Err(rc) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))),
        };

        // 2) bytes al bucket; si falla, revertir la metadata (sin huérfanos)
        if let Err(e) = storage.upload(&key, &data, &mime).await {
            error!("POST archivos ← S3 upload falló: {}", e);
            let _ = dal::baja(&state.postgres, archivo_id, tenant_id).await;
            return (StatusCode::BAD_GATEWAY, Json(json!({ "mensaje": "No se pudo subir el archivo al almacenamiento." })));
        }

        subidos.push(json!({ "id": archivo_id, "nombre": fname, "bytes": data.len(), "mime": mime }));
    }

    if subidos.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "No se recibió ningún archivo." })));
    }
    info!("POST /operaciones/proyectos/{}/archivos ← 201 {} archivo(s)", id, subidos.len());
    (StatusCode::CREATED, Json(json!({ "archivos": subidos })))
}

// ─────────────────────────────────────────────
// LISTAR — metadata + URL presignada (1h)
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/{id}/archivos",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Archivos del proyecto con URLs temporales", body = Value),
        (status = 503, description = "Almacenamiento no configurado",             body = Value),
    ),
    tag = "Archivos"
)]
pub async fn listar(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let lista = match dal::lista_proyecto(&state.postgres, tenant_id, id).await {
        Ok(l) => l,
        Err(rc) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))),
    };

    let mut archivos = Vec::with_capacity(lista.len());
    for a in &lista {
        let url = storage.presigned_get(&a.s3_key, PRESIGN_GALLERY_SECS).await.unwrap_or_default();
        archivos.push(json!({
            "id": a.id, "nombre": a.nombre, "mime": a.mime, "bytes": a.bytes,
            "area": keys::area_from_key(&a.s3_key),
            "url": url, "created_at": a.created_at.to_rfc3339(),
        }));
    }
    (StatusCode::OK, Json(json!({ "proyecto_id": id, "archivos": archivos })))
}

// ─────────────────────────────────────────────
// BORRAR — bucket + metadata
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/operaciones/archivos/{id}",
    params(("id" = i32, Path, description = "Id del archivo")),
    responses(
        (status = 200, description = "Archivo eliminado",              body = Value),
        (status = 404, description = "Archivo no encontrado",          body = Value),
        (status = 503, description = "Almacenamiento no configurado",  body = Value),
    ),
    tag = "Archivos"
)]
pub async fn borrar(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    match dal::baja(&state.postgres, id, tenant_id).await {
        Ok(Some(key)) => {
            if let Err(e) = storage.delete_object(&key).await {
                // metadata ya borrada; objeto huérfano en bucket — solo warn
                warn!("DELETE archivo {} ← objeto S3 no borrado: {}", id, e);
            }
            info!("DELETE /operaciones/archivos/{} ← 200", id);
            (StatusCode::OK, Json(json!({ "mensaje": "Archivo eliminado." })))
        }
        Ok(None) => (StatusCode::NOT_FOUND, Json(json!({ "mensaje": "Archivo no encontrado." }))),
        Err(rc)  => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))),
    }
}

// ─────────────────────────────────────────────
// USO — bytes usados vs cuota
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/storage/uso",
    responses((status = 200, description = "Uso de almacenamiento del tenant", body = Value)),
    tag = "Archivos"
)]
pub async fn uso(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };
    match dal::uso(&state.postgres, tenant_id).await {
        Ok((usados, max)) => (StatusCode::OK, Json(json!({ "usados_bytes": usados, "max_bytes": max }))),
        Err(rc) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))),
    }
}

// ─────────────────────────────────────────────
// SOPORTE — capturas de errores → support/{tenant}/{ts}/
//   No cuenta para la cuota del tenant ni se registra en cpa_tenant_archivos.
//   Tras subir, notifica por correo (payments_backend) con links presignados
//   de 7 días — best-effort, nunca bloquea la respuesta.
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/sistema/soporte",
    responses(
        (status = 200, description = "Reporte de soporte recibido",   body = Value),
        (status = 400, description = "Datos inválidos",               body = Value),
        (status = 503, description = "Almacenamiento no configurado", body = Value),
    ),
    tag = "Soporte"
)]
pub async fn soporte(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let storage = match storage_or_503(&state) { Ok(s) => s, Err(e) => return e };
    let tenant_id = match auth_user.tenant_uuid() { Ok(t) => t, Err(e) => return e };

    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let mut asunto = String::new();
    let mut descripcion = String::new();
    let mut severidad = String::from("media");
    let mut email = String::new();
    let mut keys_subidas: Vec<String> = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if let Some(fname) = field.file_name().map(str::to_string) {
            // archivo: solo imágenes, ≤5MB, máx 5
            let mime = field.content_type().unwrap_or("").to_string();
            if !mime.starts_with("image/") {
                return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Solo se aceptan imágenes en soporte." })));
            }
            let data = match field.bytes().await {
                Ok(b) => b,
                Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Imagen inválida o demasiado grande." }))),
            };
            if data.len() > MAX_SUPPORT_IMG_BYTES {
                return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": format!("'{fname}' excede 5MB.") })));
            }
            if keys_subidas.len() >= MAX_SUPPORT_IMGS {
                continue; // ignora extras silenciosamente (el form ya limita a 5)
            }
            let key = keys::support_file(&tenant_id, &ts, &fname);
            if let Err(e) = storage.upload(&key, &data, &mime).await {
                error!("POST /sistema/soporte ← S3 upload falló: {}", e);
                return (StatusCode::BAD_GATEWAY, Json(json!({ "mensaje": "No se pudieron subir las capturas." })));
            }
            keys_subidas.push(key);
        } else {
            let val = field.text().await.unwrap_or_default();
            match name.as_str() {
                "asunto"      => asunto = val,
                "descripcion" => descripcion = val,
                "severidad"   => severidad = val,
                "email"       => email = val,
                _ => {}
            }
        }
    }

    if asunto.trim().is_empty() || descripcion.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "Asunto y descripción son requeridos." })));
    }

    // Links presignados (7 días) para el correo de soporte.
    let mut links: Vec<String> = Vec::new();
    for k in &keys_subidas {
        if let Ok(u) = storage.presigned_get(k, PRESIGN_SUPPORT_SECS).await { links.push(u); }
    }

    // Acuse de recibo al USUARIO que reportó (best-effort, fire-and-forget):
    // "recibimos tu reporte y estamos trabajando en ello". Vía Outlook/Graph.
    let acuse_to = if email.trim().is_empty() { auth_user.username.clone() } else { email.clone() };
    if !acuse_to.trim().is_empty() {
        let asunto_acuse = asunto.clone();
        tokio::spawn(async move {
            let vars = [("asunto", asunto_acuse.as_str())];
            match crate::infrastructure::email::outlook::send_template(&acuse_to, "support_received", &vars).await {
                Ok(_)  => info!(%acuse_to, "soporte: acuse al usuario enviado"),
                Err(e) => warn!(%acuse_to, error = %e, "soporte: acuse al usuario falló (reporte OK)"),
            }
        });
    }

    // Notificación best-effort vía payments_backend (Outlook). No bloquea.
    if let Ok(endpoint) = std::env::var("SUPPORT_NOTIFY_ENDPOINT") {
        let payload = json!({
            "asunto":      asunto,
            "descripcion": descripcion,
            "severidad":   severidad,
            "email":       if email.is_empty() { auth_user.username.clone() } else { email.clone() },
            "tenant_id":   tenant_id.to_string(),
            "usuario":     auth_user.username,
            "links":       links,
        });
        tokio::spawn(async move {
            match reqwest::Client::new().post(&endpoint).json(&payload).send().await {
                Ok(r) if r.status().is_success() => info!("soporte: notificación enviada"),
                Ok(r)  => warn!(status = %r.status(), "soporte: notificación rechazada"),
                Err(e) => warn!(error = %e, "soporte: notificación falló"),
            }
        });
    } else {
        warn!("SUPPORT_NOTIFY_ENDPOINT no configurado — reporte guardado sin correo");
    }

    info!("POST /sistema/soporte ← 200 tenant={} imgs={}", tenant_id, keys_subidas.len());
    (StatusCode::OK, Json(json!({ "mensaje": "Reporte recibido.", "imagenes": keys_subidas.len() })))
}
