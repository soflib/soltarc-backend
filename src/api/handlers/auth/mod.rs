pub mod identity;
pub mod tokens;
pub mod sessions;
pub mod users;
pub mod roles;
pub mod tenants;
pub mod tenant_secrets;

use axum::http::StatusCode;
use serde_json::{json, Value};

pub(super) fn grpc_to_http(status: tonic::Status) -> (StatusCode, Value) {
    use tonic::Code;
    let http_status = match status.code() {
        Code::NotFound          => StatusCode::NOT_FOUND,
        Code::InvalidArgument   => StatusCode::BAD_REQUEST,
        Code::Unauthenticated   => StatusCode::UNAUTHORIZED,
        Code::PermissionDenied  => StatusCode::FORBIDDEN,
        Code::Unimplemented     => StatusCode::NOT_IMPLEMENTED,
        Code::AlreadyExists     => StatusCode::CONFLICT,
        _                       => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (http_status, json!({ "error": status.message() }))
}
