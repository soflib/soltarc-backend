pub mod xlsx;
pub mod pdf;

use axum::{
    body::Bytes,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

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
