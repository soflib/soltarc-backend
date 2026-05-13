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
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::domain::models::gn_usuarios::GnUsuarios;
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
    Json(body): Json<GnUsuariosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(user_id = %body.user_id, "POST /sistema/usuarios");

    let usr = to_model(body);
    let ret = svc::alta(&state.postgres, &usr).await;

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /sistema/usuarios/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

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
    Json(body): Json<GnUsuariosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /sistema/usuarios");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let usr = to_model(body);
    let ret = svc::cambios(&state.postgres, &usr).await;

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /sistema/usuarios/{}", id);

    match svc::consulta(&state.postgres, id).await {
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
        (status = 200, description = "Lista de usuarios",      body = Value),
        (status = 404, description = "Sin usuarios",           body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "SistemaUsuarios"
)]
pub async fn obtiene_todo(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /sistema/usuarios");

    match svc::obtiene_todo(&state.postgres).await {
        Ok(lista) => {
            info!("GET /sistema/usuarios ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(usuario_json).collect();
            (StatusCode::OK, Json(json!({ "usuarios": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /sistema/usuarios ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
