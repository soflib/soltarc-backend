// Programa...: handler::clients::dashboard
// Descripción: Dashboard del portal de clientes
// Origen.....: Cte_Inicial.aspx.cs
//
// Rutas:
//   GET /clients/portal/dashboard           → llena_det_proyectos (?grupo=&usuario=&nivel=)
//   GET /clients/portal/projects/{id}/ppto  → total_ppto

use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::api::middleware::roles::AuthUser;
use crate::services::clients::dashboard as svc;

// ─────────────────────────────────────────────
// LLENA DET PROYECTOS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/clients/portal/dashboard",
    responses(
        (status = 200, description = "Proyectos visibles para el usuario logueado", body = Value),
        (status = 404, description = "Sin proyectos",                  body = Value),
        (status = 500, description = "Error de base de datos",         body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn llena_det_proyectos(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/dashboard (user={})", auth_user.user_id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    // Visibilidad por el PERFIL REAL del usuario logueado (NO params del front):
    // Admin → nivel 1 (ve todo); el resto según su perfil de negocio
    // (1=todo, 2=su grupo, 3=solo lo asignado a él). Sin perfil → nivel 1.
    let (grupo, gn_usr_id, nivel) = if auth_user.role.eq_ignore_ascii_case("Admin") {
        (0, 0, 1)
    } else {
        match uuid::Uuid::parse_str(&auth_user.user_id) {
            Ok(uid) => crate::dal::gn_usuarios::perfil_de(&state.postgres, tenant_id, uid).await,
            Err(_)  => (0, 0, 1),
        }
    };

    // Alcance real: nivel 2 necesita un grupo asignado (>0); nivel 3 necesita una
    // asignación directa (>0). Sin asignación (0) NO ve nada — no debe "caer" a los
    // proyectos sin grupo/usuario (gn_id/gn_usr_id = 0). Solo Admin/nivel 1 ve todo.
    let con_alcance = match nivel {
        1 => true,
        2 => grupo > 0,
        _ => gn_usr_id > 0,
    };
    if !con_alcance {
        info!("GET /clients/portal/dashboard ← 200 0 proyectos (usuario sin asignación)");
        return (StatusCode::OK, Json(json!([])));
    }

    match svc::llena_det_proyectos(&state.postgres, tenant_id, grupo, gn_usr_id, nivel).await {
        Ok(lista) => {
            info!("GET /clients/portal/dashboard ← 200 {} proyectos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|p| json!({
                "proyecto_id":    p.proyecto_id,
                "nombre":         p.nombre,
                "presupuesto":    p.presupuesto.as_ref().map(|d| d.to_string()),
                "total_ingresos": p.total_ingresos.as_ref().map(|d| d.to_string()),
                "total_egresos":  p.total_egresos.as_ref().map(|d| d.to_string()),
                "saldo":          p.saldo.as_ref().map(|d| d.to_string()),
                "estado":         p.estado,
                "cliente":        p.cliente,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/dashboard ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/dashboard ← 404 sin proyectos");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// TOTAL PPTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/clients/portal/projects/{id}/ppto",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Total presupuesto", body = Value),
        (status = 404, description = "Sin datos",         body = Value),
        (status = 500, description = "Error BD",          body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn total_ppto(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/projects/{}/ppto", id);
    if let Some(err) = super::ensure_proyecto(&state, &auth_user, id).await { return err; }

    match svc::total_ppto(&state.postgres, id).await {
        Ok(total) => {
            info!("GET /clients/portal/projects/{}/ppto ← 200 total={}", id, total);
            (StatusCode::OK, Json(json!({ "proyecto_id": id, "total_ppto": total.to_string() })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/projects/{}/ppto ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/projects/{}/ppto ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
