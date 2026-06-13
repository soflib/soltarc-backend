// Programa...: handler::clients::account_statement
// Descripción: Estado de cuenta del portal de clientes
// Origen.....: Cte_Estado_De_Cuenta.aspx.cs
//
// Rutas:
//   GET /clients/portal/clients/{id}/account-statement → estado_de_cuenta + nombre_cliente

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;
use crate::services::clients::account_statement as svc;

#[utoipa::path(
    get,
    path = "/clients/portal/clients/{id}/account-statement",
    params(("id" = i32, Path, description = "Id del cliente")),
    responses(
        (status = 200, description = "Estado de cuenta del cliente", body = Value),
        (status = 404, description = "Sin movimientos",              body = Value),
        (status = 500, description = "Error de base de datos",       body = Value),
    ),
    tag = "Client Portal"
)]
pub async fn account_statement(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/clients/{}/account-statement", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    // Acceso por perfil: un no-admin solo puede ver el estado de cuenta de
    // clientes que pertenecen a sus proyectos visibles (su grupo/asignación).
    let (grupo, gn_usr_id, nivel) = perfil(&state, &auth_user, tenant_id).await;
    if !crate::dal::clientes::cliente_accesible(&state.postgres, tenant_id, grupo, gn_usr_id, nivel, id).await {
        info!("GET /clients/portal/clients/{}/account-statement ← 403 (sin acceso al cliente)", id);
        return (StatusCode::FORBIDDEN, Json(json!({ "mensaje": "No tienes acceso a este cliente." })));
    }

    let nombre_ret = svc::nombre_cliente(&state.postgres, id, tenant_id).await;

    match svc::estado_de_cuenta(&state.postgres, id).await {
        Ok(movimientos) => {
            info!(
                "GET /clients/portal/clients/{}/account-statement ← 200 {} movimientos",
                id, movimientos.len()
            );
            (StatusCode::OK, Json(json!({
                "cliente_id":     id,
                "nombre_cliente": nombre_ret.mensaje,
                "movimientos": movimientos.iter().map(|m| json!({
                    "fecha":      m.fecha,
                    "concepto":   m.concepto,
                    "referencia": m.referencia,
                    "cargo":      m.cargo.as_ref().map(|d| d.to_string()),
                    "abono":      m.abono.as_ref().map(|d| d.to_string()),
                    "saldo":      m.saldo.as_ref().map(|d| d.to_string()),
                })).collect::<Vec<_>>(),
            })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /clients/portal/clients/{}/account-statement ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /clients/portal/clients/{}/account-statement ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// Perfil de acceso del usuario logueado: Admin → (0,0,1) = ve todo; el resto
// según su gn_usuarios (grupo, gn_usr_id, nivel); sin perfil → (0,0,1).
async fn perfil(state: &AppState, auth_user: &AuthUser, tenant_id: uuid::Uuid) -> (i32, i32, i32) {
    if auth_user.role.eq_ignore_ascii_case("Admin") {
        (0, 0, 1)
    } else {
        match uuid::Uuid::parse_str(&auth_user.user_id) {
            Ok(uid) => crate::dal::gn_usuarios::perfil_de(&state.postgres, tenant_id, uid).await,
            Err(_)  => (0, 0, 1),
        }
    }
}

// ─────────────────────────────────────────────
// MIS CLIENTES — clientes de los proyectos visibles del usuario (para el combo
// de Estado de Cuenta). Así el no-admin no puede ni elegir clientes ajenos.
//   GET /clients/portal/clients
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/clients/portal/clients",
    responses((status = 200, description = "Clientes accesibles para el usuario", body = Value)),
    tag = "Client Portal"
)]
pub async fn mis_clientes(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };
    let (grupo, gn_usr_id, nivel) = perfil(&state, &auth_user, tenant_id).await;
    let lista = crate::dal::clientes::clientes_accesibles(&state.postgres, tenant_id, grupo, gn_usr_id, nivel).await;
    debug!("GET /clients/portal/clients ← 200 {} clientes (nivel {})", lista.len(), nivel);
    (StatusCode::OK, Json(json!(lista.iter()
        .map(|c| json!({ "id": c.id, "nombre": c.etiqueta }))
        .collect::<Vec<_>>())))
}
