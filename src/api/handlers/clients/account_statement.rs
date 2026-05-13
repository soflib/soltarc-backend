// Programa...: handler::clients::account_statement
// Descripción: Estado de cuenta del portal de clientes
// Origen.....: Cte_Estado_De_Cuenta.aspx.cs
//
// Rutas:
//   GET /clients/portal/clients/{id}/account-statement → estado_de_cuenta + nombre_cliente

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /clients/portal/clients/{}/account-statement", id);

    let nombre_ret = svc::nombre_cliente(&state.postgres, id).await;

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
