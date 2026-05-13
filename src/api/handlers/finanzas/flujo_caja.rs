// Programa...: handler::finanzas::flujo_caja
// Origen.....: oFlujoCaja.cs
//
// Rutas:
//   GET /finanzas/flujo-caja?fecha_saldo=YYYY-MM-DD&fecha_ini=YYYY-MM-DD&fecha_fin=YYYY-MM-DD&format=

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use time::macros::format_description;
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::render;
use crate::services::finanzas::flujo_caja as svc;

#[derive(Debug, Deserialize)]
pub struct FlujoCajaQuery {
    pub fecha_saldo: String,
    pub fecha_ini:   String,
    pub fecha_fin:   String,
    pub format:      Option<String>,
}

fn parse_date(s: &str) -> Result<time::Date, String> {
    let fmt = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, &fmt).map_err(|e| format!("fecha inválida '{}': {}", s, e))
}

#[utoipa::path(
    get,
    path = "/finanzas/flujo-caja",
    params(
        ("fecha_saldo" = String,         Query, description = "Fecha de saldo base YYYY-MM-DD"),
        ("fecha_ini"   = String,         Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin"   = String,         Query, description = "Fin del período YYYY-MM-DD"),
        ("format"      = Option<String>, Query, description = "xlsx"),
    ),
    responses(
        (status = 200, description = "Flujo de caja",          body = Value),
        (status = 400, description = "Parámetros inválidos",   body = Value),
        (status = 404, description = "Sin movimientos",        body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn consulta_flujo(
    State(state): State<AppState>,
    Query(q): Query<FlujoCajaQuery>,
) -> Response {
    debug!("GET /finanzas/flujo-caja fecha_saldo={}", q.fecha_saldo);

    let fecha_saldo = match parse_date(&q.fecha_saldo) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::consulta_flujo(&state.postgres, fecha_saldo, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /finanzas/flujo-caja ← 200 {} registros", lista.len());
            let total: rust_decimal::Decimal = lista.iter().map(|f| f.monto).sum();
            let items: Vec<Value> = lista.iter().map(|f| json!({
                "tipo":  f.tipo,
                "banco": f.banco,
                "monto": f.monto.to_string(),
            })).collect();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::flujo_caja(&items) {
                    Ok(b)  => render::xlsx_resp(b, "flujo_caja.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "flujo": items, "total": total.to_string() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/flujo-caja ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /finanzas/flujo-caja ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}
