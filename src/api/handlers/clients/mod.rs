pub mod account_statement;
pub mod dashboard;
pub mod expense_detail;
pub mod project_images;
pub mod project_tasks;
pub mod project_tree;
pub mod weekly_plan;
pub mod work_progress;

use axum::{http::StatusCode, Json};
use serde_json::{json, Value};
use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;

/// Acceso a un proyecto del portal: Admin ve todo; el resto solo SUS proyectos
/// (su grupo / asignación). Devuelve Some(respuesta de error) si NO puede verlo;
/// None si sí. Usar al inicio de cada handler `/clients/portal/projects/{id}/*`.
pub async fn ensure_proyecto(
    state: &AppState,
    auth_user: &AuthUser,
    proyecto: i32,
) -> Option<(StatusCode, Json<Value>)> {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return Some(e),
    };
    let (grupo, gn_usr_id, nivel) = if auth_user.role.eq_ignore_ascii_case("Admin") {
        (0, 0, 1)
    } else {
        match uuid::Uuid::parse_str(&auth_user.user_id) {
            Ok(uid) => crate::dal::gn_usuarios::perfil_de(&state.postgres, tenant_id, uid).await,
            Err(_)  => (0, 0, 1),
        }
    };
    if crate::dal::proyectos::proyecto_accesible(&state.postgres, tenant_id, grupo, gn_usr_id, nivel, proyecto).await {
        None
    } else {
        Some((StatusCode::FORBIDDEN, Json(json!({ "mensaje": "No tienes acceso a este proyecto." }))))
    }
}
