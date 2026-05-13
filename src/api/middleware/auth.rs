use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::infrastructure::db::app_state::AppState;
use crate::generated::auth::ValidateRequest;
use crate::api::middleware::roles::AuthUser;

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(v) if v.starts_with("Bearer ") => v[7..].to_string(),
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "missing or malformed Authorization header" })),
            )
                .into_response();
        }
    };

    let mut grpc = state.auth_grpc.clone();
    match grpc.validate_token(ValidateRequest { access_token: token }).await {
        Ok(r) if r.valid => {
            match r.user {
                Some(payload) => {
                    request.extensions_mut().insert(AuthUser {
                        user_id:   payload.user_id,
                        username:  payload.username,
                        role:      payload.role,
                        tenant_id: payload.tenant_id,
                    });
                    next.run(request).await
                }
                None => (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "invalid token claims" })),
                )
                    .into_response(),
            }
        }
        Ok(r) => {
            let msg = if r.error.is_empty() { "invalid token".into() } else { r.error };
            (StatusCode::UNAUTHORIZED, Json(json!({ "error": msg }))).into_response()
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "token validation failed" })),
        )
            .into_response(),
    }
}
