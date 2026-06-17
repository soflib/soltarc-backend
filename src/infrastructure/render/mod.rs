pub mod xlsx;
pub mod pdf;

use axum::{
    body::Bytes,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;

/// Baja los bytes del logo del tenant desde Contabo para insertarlo en los PDFs.
/// Best-effort: devuelve None si no hay logo o el storage no está configurado,
/// de modo que el reporte se genera igual (sin logo).
pub async fn tenant_logo_bytes(state: &AppState, auth_user: &AuthUser) -> Option<Vec<u8>> {
    let storage = state.storage.clone()?;
    let tid = auth_user.tenant_uuid().ok()?;
    let key = crate::infrastructure::storage::contabo::keys::tenant_logo(&tid);
    storage.download(&key).await.ok()
}

pub fn xlsx_resp(bytes: Vec<u8>, filename: &str) -> Response {
    let mut h = HeaderMap::new();
    h.insert(
        header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            .parse()
            .unwrap(),
    );
    h.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
    );
    (StatusCode::OK, h, Bytes::from(bytes)).into_response()
}

pub fn pdf_resp(bytes: Vec<u8>, filename: &str) -> Response {
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, "application/pdf".parse().unwrap());
    h.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename).parse().unwrap(),
    );
    (StatusCode::OK, h, Bytes::from(bytes)).into_response()
}

pub fn render_err(msg: String) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "codigo": -1, "mensaje": msg })),
    )
        .into_response()
}
