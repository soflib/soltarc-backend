// Programa...: handler::reportes::lookups
// Descripción: Autocompletes para la página de Reportes, SCOPEADOS por el perfil
//              del usuario (Admin/nivel1 = todo; nivel 2 = su grupo; nivel 3 =
//              lo asignado). Así un rol como "reportes" solo ve SUS entidades.
//
// Rutas:
//   GET /reportes/lookup/proyectos?q=&limit=
//   GET /reportes/lookup/clientes?q=&limit=
//   GET /reportes/lookup/presupuestos?q=&limit=

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::middleware::roles::AuthUser;
use crate::dal::gn_usuarios::perfil_de_auth;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::app_state::AppState;

#[derive(Debug, Deserialize)]
pub struct LookupQuery {
    pub q:     Option<String>,
    pub limit: Option<i32>,
}

// (tenant, grupo, gn_usr_id, nivel) del usuario logueado.
async fn perfil(
    state: &AppState,
    auth_user: &AuthUser,
) -> Result<(uuid::Uuid, i32, i32, i32), (StatusCode, Json<Value>)> {
    let tenant_id = auth_user.tenant_uuid()?;
    let (g, u, n) = perfil_de_auth(&state.postgres, tenant_id, &auth_user.user_id, &auth_user.role).await;
    Ok((tenant_id, g, u, n))
}

fn to_json(items: Vec<LookupItem>) -> Json<Value> {
    Json(json!(items
        .iter()
        .map(|i| json!({ "id": i.id, "etiqueta": i.etiqueta }))
        .collect::<Vec<_>>()))
}

pub async fn proyectos(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<LookupQuery>,
) -> (StatusCode, Json<Value>) {
    let (tenant, g, u, n) = match perfil(&state, &auth_user).await { Ok(p) => p, Err(e) => return e };
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let items = crate::dal::proyectos::lookup_accesibles(
        &state.postgres, tenant, g, u, n, &q.q.unwrap_or_default(), None, limit,
    ).await;
    (StatusCode::OK, to_json(items))
}

pub async fn clientes(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<LookupQuery>,
) -> (StatusCode, Json<Value>) {
    let (tenant, g, u, n) = match perfil(&state, &auth_user).await { Ok(p) => p, Err(e) => return e };
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let items = crate::dal::clientes::lookup_accesibles(
        &state.postgres, tenant, g, u, n, &q.q.unwrap_or_default(), limit,
    ).await;
    (StatusCode::OK, to_json(items))
}

pub async fn presupuestos(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<LookupQuery>,
) -> (StatusCode, Json<Value>) {
    let (tenant, g, u, n) = match perfil(&state, &auth_user).await { Ok(p) => p, Err(e) => return e };
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let items = crate::dal::presupuesto::lookup_accesibles(
        &state.postgres, tenant, g, u, n, &q.q.unwrap_or_default(), limit,
    ).await;
    (StatusCode::OK, to_json(items))
}
