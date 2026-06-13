// Programa...: handler::reportes::financieros
// Origen.....: oReportes.cs
//
// Rutas:
//   GET /reportes/financieros/captura-diaria?fecha_ini=&fecha_fin=[&format=xlsx]
//   GET /reportes/financieros/ingresos?fecha_ini=&fecha_fin=[&format=xlsx]
//   GET /reportes/financieros/ingresos-cliente?id=&fecha_ini=&fecha_fin=[&format=xlsx]
//   GET /reportes/financieros/egresos-centros-costo?id=&fecha_ini=&fecha_fin=[&format=xlsx]
//   GET /reportes/financieros/egresos-proveedor?id=&fecha_ini=&fecha_fin=[&format=xlsx]
//   GET /reportes/financieros/egresos?fecha_ini=&fecha_fin=[&format=xlsx]
//   GET /reportes/financieros/egresos-gral?banco=&fecha_ini=&fecha_fin=[&format=xlsx]

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use time::macros::format_description;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::generated::auth::GetAllUsersRequest;
use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::render;
use crate::services::reportes::financieros as svc;

fn parse_date(s: &str) -> Result<time::Date, String> {
    let fmt = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, &fmt).map_err(|e| format!("fecha inválida '{}': {}", s, e))
}

const UUID_ZERO: &str = "00000000-0000-0000-0000-000000000000";

// Mapa user_id (UUID) → nombre legible, vía el servicio de auth (gRPC).
// El nombre del usuario vive en security_db (otra BD), por eso NO se puede
// resolver con un JOIN en el SP; se resuelve aquí. Si el gRPC falla, devuelve
// un mapa vacío y los reportes muestran el UUID crudo (degradación suave).
async fn mapa_usuarios(state: &AppState, tenant_id: &str) -> HashMap<String, String> {
    if tenant_id.is_empty() {
        return HashMap::new();
    }
    let mut client = state.auth_grpc.clone();
    let req = GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: tenant_id.to_string() };
    match client.get_all_users(req).await {
        Ok(r) => r.users.into_iter().map(|u| {
            let nombre = if !u.full_name.trim().is_empty() { u.full_name }
                         else if !u.username.trim().is_empty() { u.username }
                         else { u.email };
            (u.user_id, nombre)
        }).collect(),
        Err(_) => HashMap::new(),
    }
}

// Resuelve el UUID del usuario a un nombre legible. El UUID cero (datos de
// ejemplo / sistema) se muestra como "Sistema"; un UUID desconocido cae al
// propio UUID para no perder información.
fn resolver_usuario(uuid: Option<&str>, mapa: &HashMap<String, String>) -> String {
    match uuid {
        None => String::new(),
        Some(u) if u.is_empty() || u == UUID_ZERO => "Sistema".to_string(),
        Some(u) => mapa.get(u).cloned().unwrap_or_else(|| u.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct FechasQuery {
    pub fecha_ini: String,
    pub fecha_fin: String,
    pub format:    Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdFechasQuery {
    pub id:        i32,
    pub fecha_ini: String,
    pub fecha_fin: String,
    pub format:    Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BancoFechasQuery {
    pub banco:     i32,
    pub fecha_ini: String,
    pub fecha_fin: String,
    pub format:    Option<String>,
}

// ── Captura diaria ────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/captura-diaria",
    params(
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Captura diaria del período", body = Value),
        (status = 400, description = "Fechas inválidas",           body = Value),
        (status = 404, description = "Sin registros",              body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "Reportes"
)]
pub async fn captura_diaria(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<FechasQuery>,
) -> Response {
    debug!("GET /reportes/financieros/captura-diaria");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::captura_diaria(&state.postgres, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/captura-diaria ← 200 {} registros", lista.len());
            // Resuelve los UUID de usuario a nombres legibles (gRPC auth).
            let usuarios = mapa_usuarios(&state, &auth_user.tenant_id).await;
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "tipo":       r.tipo,
                "fecha":      r.fecha.map(|d| d.to_string()),
                "banco":      r.banco,
                "cuenta":     r.cuenta,
                "referencia": r.referencia,
                "concepto":   r.concepto,
                "monto":      r.monto.map(|m| m.to_string()),
                "usuario":    resolver_usuario(r.usuario.as_deref(), &usuarios),
                "proyecto":   r.proyecto,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::captura_diaria(&items) {
                    Ok(b)  => render::xlsx_resp(b, "captura_diaria.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "registros": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/captura-diaria ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Ingresos reporte ──────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/ingresos",
    params(
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Reporte de ingresos del período", body = Value),
        (status = 400, description = "Fechas inválidas",                body = Value),
        (status = 404, description = "Sin registros",                   body = Value),
        (status = 500, description = "Error de base de datos",          body = Value),
    ),
    tag = "Reportes"
)]
pub async fn ingresos_reporte(
    State(state): State<AppState>,
    Query(q): Query<FechasQuery>,
) -> Response {
    debug!("GET /reportes/financieros/ingresos");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::ingresos_reporte(&state.postgres, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/ingresos ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "id":         r.id,
                "fecha":      r.fecha.map(|d| d.to_string()),
                "banco":      r.banco,
                "cuenta":     r.cuenta,
                "forma_pago": r.forma_pago,
                "referencia": r.referencia,
                "cliente":    r.cliente,
                "proyecto":   r.proyecto,
                "monto":      r.monto.map(|m| m.to_string()),
                "comentario": r.comentario,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::ingresos_reporte(&items) {
                    Ok(b)  => render::xlsx_resp(b, "ingresos_reporte.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "ingresos": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/ingresos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Ingresos por cliente ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/ingresos-cliente",
    params(
        ("id"        = i32,    Query, description = "Id del cliente"),
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Ingresos del cliente en el período", body = Value),
        (status = 400, description = "Fechas inválidas",                   body = Value),
        (status = 404, description = "Sin registros",                      body = Value),
        (status = 500, description = "Error de base de datos",             body = Value),
    ),
    tag = "Reportes"
)]
pub async fn ingresos_cliente(
    State(state): State<AppState>,
    Query(q): Query<IdFechasQuery>,
) -> Response {
    debug!(id = q.id, "GET /reportes/financieros/ingresos-cliente");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::ingresos_cliente(&state.postgres, q.id, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/ingresos-cliente?id={} ← 200 {} registros", q.id, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "id":         r.id,
                "fecha":      r.fecha.map(|d| d.to_string()),
                "banco":      r.banco,
                "cuenta":     r.cuenta,
                "forma_pago": r.forma_pago,
                "referencia": r.referencia,
                "proyecto":   r.proyecto,
                "monto":      r.monto.map(|m| m.to_string()),
                "comentario": r.comentario,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::ingresos_cliente(&items) {
                    Ok(b)  => render::xlsx_resp(b, "ingresos_cliente.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "ingresos": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/ingresos-cliente ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Egresos por centros de costo ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/egresos-centros-costo",
    params(
        ("id"        = i32,    Query, description = "Id del centro de costo"),
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Egresos del centro de costo en el período", body = Value),
        (status = 400, description = "Fechas inválidas",                          body = Value),
        (status = 404, description = "Sin registros",                             body = Value),
        (status = 500, description = "Error de base de datos",                    body = Value),
    ),
    tag = "Reportes"
)]
pub async fn egresos_centros_costo(
    State(state): State<AppState>,
    Query(q): Query<IdFechasQuery>,
) -> Response {
    debug!(id = q.id, "GET /reportes/financieros/egresos-centros-costo");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::egresos_centros_costo(&state.postgres, q.id, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/egresos-centros-costo?id={} ← 200 {} registros", q.id, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "id":           r.id,
                "fecha":        r.fecha.map(|d| d.to_string()),
                "banco":        r.banco,
                "cuenta":       r.cuenta,
                "forma_pago":   r.forma_pago,
                "referencia":   r.referencia,
                "proyecto":     r.proyecto,
                "proveedor":    r.proveedor,
                "monto":        r.monto.map(|m| m.to_string()),
                "comentario":   r.comentario,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::egresos_centros_costo(&items) {
                    Ok(b)  => render::xlsx_resp(b, "egresos_centros_costo.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "egresos": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/egresos-centros-costo ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Egresos por proveedor ─────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/egresos-proveedor",
    params(
        ("id"        = i32,    Query, description = "Id del proveedor"),
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Egresos del proveedor en el período", body = Value),
        (status = 400, description = "Fechas inválidas",                    body = Value),
        (status = 404, description = "Sin registros",                       body = Value),
        (status = 500, description = "Error de base de datos",              body = Value),
    ),
    tag = "Reportes"
)]
pub async fn egresos_proveedor(
    State(state): State<AppState>,
    Query(q): Query<IdFechasQuery>,
) -> Response {
    debug!(id = q.id, "GET /reportes/financieros/egresos-proveedor");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::egresos_proveedor(&state.postgres, q.id, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/egresos-proveedor?id={} ← 200 {} registros", q.id, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "id":           r.id,
                "fecha":        r.fecha.map(|d| d.to_string()),
                "banco":        r.banco,
                "cuenta":       r.cuenta,
                "forma_pago":   r.forma_pago,
                "referencia":   r.referencia,
                "proyecto":     r.proyecto,
                "centro_costo": r.centro_costo,
                "monto":        r.monto.map(|m| m.to_string()),
                "comentario":   r.comentario,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::egresos_proveedor(&items) {
                    Ok(b)  => render::xlsx_resp(b, "egresos_proveedor.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "egresos": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/egresos-proveedor ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Egresos reporte por período ───────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/egresos",
    params(
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Reporte de egresos del período", body = Value),
        (status = 400, description = "Fechas inválidas",               body = Value),
        (status = 404, description = "Sin registros",                  body = Value),
        (status = 500, description = "Error de base de datos",         body = Value),
    ),
    tag = "Reportes"
)]
pub async fn egresos_reporte(
    State(state): State<AppState>,
    Query(q): Query<FechasQuery>,
) -> Response {
    debug!("GET /reportes/financieros/egresos");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::egresos_reporte(&state.postgres, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/egresos ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "id":           r.id,
                "fecha":        r.fecha.map(|d| d.to_string()),
                "banco":        r.banco,
                "cuenta":       r.cuenta,
                "forma_pago":   r.forma_pago,
                "referencia":   r.referencia,
                "proyecto":     r.proyecto,
                "proveedor":    r.proveedor,
                "centro_costo": r.centro_costo,
                "monto":        r.monto.map(|m| m.to_string()),
                "comentario":   r.comentario,
                "usuario":      r.usuario,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::egresos_reporte(&items) {
                    Ok(b)  => render::xlsx_resp(b, "egresos.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "egresos": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/egresos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Reporte general de egresos por banco ─────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/financieros/egresos-gral",
    params(
        ("banco"     = i32,    Query, description = "Id del banco"),
        ("fecha_ini" = String, Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String, Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx para exportar"),
    ),
    responses(
        (status = 200, description = "Reporte general de egresos del banco", body = Value),
        (status = 400, description = "Fechas inválidas",                     body = Value),
        (status = 404, description = "Sin registros",                        body = Value),
        (status = 500, description = "Error de base de datos",               body = Value),
    ),
    tag = "Reportes"
)]
pub async fn reporte_gral_egresos(
    State(state): State<AppState>,
    Query(q): Query<BancoFechasQuery>,
) -> Response {
    debug!(banco = q.banco, "GET /reportes/financieros/egresos-gral");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::reporte_gral_egresos(&state.postgres, q.banco, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /reportes/financieros/egresos-gral?banco={} ← 200 {} registros", q.banco, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "id":           r.id,
                "fecha":        r.fecha.map(|d| d.to_string()),
                "banco":        r.banco,
                "cuenta":       r.cuenta,
                "forma_pago":   r.forma_pago,
                "referencia":   r.referencia,
                "proyecto":     r.proyecto,
                "proveedor":    r.proveedor,
                "centro_costo": r.centro_costo,
                "monto":        r.monto.map(|m| m.to_string()),
                "comentario":   r.comentario,
                "usuario":      r.usuario,
            })).collect();

            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::egresos_gral(&items) {
                    Ok(b)  => render::xlsx_resp(b, "egresos_gral.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "egresos": items, "total": items.len() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/financieros/egresos-gral ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}
