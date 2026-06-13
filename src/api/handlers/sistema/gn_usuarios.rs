// Programa...: handler::sistema::gn_usuarios
// Origen.....: oGNUsuarios.cs
//
// Rutas:
//   POST   /sistema/usuarios              → alta
//   DELETE /sistema/usuarios/{id}         → baja
//   PUT    /sistema/usuarios              → cambios
//   GET    /sistema/usuarios/{id}         → consulta
//   GET    /sistema/usuarios              → obtiene_todo

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};
use utoipa::ToSchema;

use uuid::Uuid;

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::gn_usuarios::GnUsuarios;
use crate::generated::auth::GetAllUsersRequest;
use crate::infrastructure::db::app_state::AppState;
use crate::services::sistema::gn_usuarios as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GnUsuariosInput {
    pub id:             Option<i32>,
    pub user_id:        String,
    pub grupo_negocio:  i32,
    pub activo:         bool,
    pub nivel:          i32,
    pub opt_cte_1:      bool,
    pub opt_cte_2:      bool,
    pub opt_cte_3:      bool,
    pub opt_cte_4:      bool,
    pub opt_cte_5:      bool,
    pub opt_cte_6:      bool,
}

fn to_model(body: GnUsuariosInput) -> GnUsuarios {
    GnUsuarios {
        id:            body.id.unwrap_or(0),
        user_id:       body.user_id,
        grupo_negocio: body.grupo_negocio,
        activo:        body.activo,
        nivel:         body.nivel,
        opt_cte_1:     body.opt_cte_1,
        opt_cte_2:     body.opt_cte_2,
        opt_cte_3:     body.opt_cte_3,
        opt_cte_4:     body.opt_cte_4,
        opt_cte_5:     body.opt_cte_5,
        opt_cte_6:     body.opt_cte_6,
    }
}

fn usuario_json(u: &GnUsuarios) -> Value {
    json!({
        "id":            u.id,
        "user_id":       u.user_id,
        "grupo_negocio": u.grupo_negocio,
        "activo":        u.activo,
        "nivel":         u.nivel,
        "opt_cte_1":     u.opt_cte_1,
        "opt_cte_2":     u.opt_cte_2,
        "opt_cte_3":     u.opt_cte_3,
        "opt_cte_4":     u.opt_cte_4,
        "opt_cte_5":     u.opt_cte_5,
        "opt_cte_6":     u.opt_cte_6,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/sistema/usuarios",
    request_body = GnUsuariosInput,
    responses(
        (status = 201, description = "Usuario registrado",        body = Value),
        (status = 400, description = "Alta cancelada o error",    body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<GnUsuariosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %body.user_id, "POST /sistema/usuarios");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let usr = to_model(body);
    let ret = svc::alta(&state.postgres, &usr, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /sistema/usuarios ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /sistema/usuarios ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/sistema/usuarios/{id}",
    params(("id" = i32, Path, description = "Id del usuario a eliminar")),
    responses(
        (status = 200, description = "Usuario eliminado",      body = Value),
        (status = 400, description = "Baja cancelada o error", body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /sistema/usuarios/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /sistema/usuarios/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambios ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/sistema/usuarios",
    request_body = GnUsuariosInput,
    responses(
        (status = 200, description = "Usuario actualizado",                  body = Value),
        (status = 400, description = "Actualización cancelada o error",      body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<GnUsuariosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /sistema/usuarios");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let usr = to_model(body);
    let ret = svc::cambios(&state.postgres, &usr, tenant_id).await;

    if ret.afectado > 0 {
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /sistema/usuarios ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/sistema/usuarios/{id}",
    params(("id" = i32, Path, description = "Id del usuario a consultar")),
    responses(
        (status = 200, description = "Usuario encontrado",     body = Value),
        (status = 404, description = "Usuario no encontrado",  body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /sistema/usuarios/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(u)) => (StatusCode::OK, Json(usuario_json(&u))),
        Ok(None)    => (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Usuario no encontrado" }))),
        Err(rc)     => {
            error!("GET /sistema/usuarios/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Obtiene todo ──────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/sistema/usuarios",
    responses(
        (status = 200, description = "Lista de usuarios (vacía si no hay registros)", body = Value),
        (status = 500, description = "Error de base de datos",                        body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn obtiene_todo(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /sistema/usuarios");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::obtiene_todo(&state.postgres, tenant_id).await {
        Ok(lista) => {
            info!("GET /sistema/usuarios ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(usuario_json).collect();
            (StatusCode::OK, Json(json!({ "usuarios": items, "total": items.len() })))
        }
        // DAL returns codigo -21 ("No hay entradas") when the table is empty.
        // Treat that as an empty list (200), not a missing resource (404), so callers
        // that combine multiple list calls don't blow up on first-time setup.
        Err(rc) if rc.codigo == -21 => {
            info!("GET /sistema/usuarios ← 200 [] (sin registros)");
            (StatusCode::OK, Json(json!({ "usuarios": [], "total": 0 })))
        }
        Err(rc) => {
            error!("GET /sistema/usuarios ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Sync con usuarios reales ────────────────────────────────────────────────
// Liga cada usuario real (auth.users) del tenant a su perfil gn_usuarios
// (usuario_uuid + nivel default por rol). Idempotente. Necesario para tenants
// existentes; los nuevos se sincronizan solos al registrar el admin.

#[utoipa::path(
    post,
    path = "/sistema/usuarios/sync",
    responses(
        (status = 200, description = "Usuarios sincronizados", body = Value),
        (status = 502, description = "No se pudo consultar auth", body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn sync(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let mut client = state.auth_grpc.clone();
    let resp = match client.get_all_users(GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: auth_user.tenant_id.clone() }).await {
        Ok(r)  => r,
        Err(e) => {
            error!("POST /sistema/usuarios/sync ← 502 {}", e);
            return (StatusCode::BAD_GATEWAY, Json(json!({ "codigo": -1, "mensaje": "No se pudo obtener usuarios de auth" })));
        }
    };
    let lista: Vec<(Uuid, String, String)> = resp.users.into_iter()
        .filter_map(|u| Uuid::parse_str(&u.user_id).ok().map(|id| (id, u.email, u.role)))
        .collect();
    let n = crate::dal::gn_usuarios::sync_for_tenant(&state.postgres, tenant_id, &lista).await;
    info!("POST /sistema/usuarios/sync ← 200 {} usuarios", n);
    (StatusCode::OK, Json(json!({ "sincronizados": n })))
}
