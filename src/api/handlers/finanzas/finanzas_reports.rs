// Programa...: handler::finanzas::finanzas_reports
// Origen.....: oFinanzas.cs + ReciboHonorarios.aspx
//
// Rutas:
//   GET /finanzas/trx?proyecto={id}
//   PUT /finanzas/egresos/{id}/distribuye
//   GET /finanzas/reportes/egresos-proveedor?tipo=bool&fecha_ini=YYYY-MM-DD&fecha_fin=YYYY-MM-DD&format=
//   GET /finanzas/reportes/ingresos-detalle?fecha_ini=YYYY-MM-DD&fecha_fin=YYYY-MM-DD&format=
//   GET /finanzas/recibo-honorarios/{id}?tipo=egreso|ingreso&format=

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use time::macros::format_description;
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::api::middleware::roles::AuthUser;
use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::render;
use crate::services::finanzas::finanzas_reports as svc;

fn parse_date(s: &str) -> Result<time::Date, String> {
    let fmt = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, &fmt).map_err(|e| format!("fecha inválida '{}': {}", s, e))
}

#[derive(Debug, Deserialize)]
pub struct TrxQuery {
    pub proyecto: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DistribuyeInput {
    pub nodo: String,
}

#[derive(Debug, Deserialize)]
pub struct EgresosProveedorQuery {
    pub tipo:      bool,
    pub fecha_ini: String,
    pub fecha_fin: String,
    pub format:    Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FechasQuery {
    pub fecha_ini: String,
    pub fecha_fin: String,
    pub format:    Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TipoQuery {
    pub tipo:   String,
    pub format: Option<String>,
}

// ── TRX financieras por proyecto ─────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/trx",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Transacciones financieras del proyecto", body = Value),
        (status = 404, description = "Sin transacciones",                      body = Value),
        (status = 500, description = "Error de base de datos",                 body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn trx_financieras(
    State(state): State<AppState>,
    Query(q): Query<TrxQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = q.proyecto, "GET /finanzas/trx");

    match svc::trx_financieras(&state.postgres, q.proyecto).await {
        Ok(lista) => {
            info!("GET /finanzas/trx?proyecto={} ← 200 {} registros", q.proyecto, lista.len());
            let items: Vec<Value> = lista.iter().map(|t| json!({
                "id":           t.id,
                "tipo":         t.tipo,
                "fecha_cap":    t.fecha_cap.map(|d| d.to_string()),
                "fecha_aplica": t.fecha_aplica.map(|d| d.to_string()),
                "banco":        t.banco,
                "cuenta":       t.cuenta,
                "forma_pago":   t.forma_pago,
                "centro_costo": t.centro_costo,
                "referencia":   t.referencia,
                "comentario":   t.comentario,
                "proyecto":     t.proyecto,
                "usuario":      t.usuario,
                "cte_pro":      t.cte_pro,
                "monto":        t.monto.map(|m| m.to_string()),
            })).collect();
            let total = items.len();
            (StatusCode::OK, Json(json!({ "transacciones": items, "total": total })))
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/trx?proyecto={} ← 404", q.proyecto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /finanzas/trx ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Distribuye egreso proporcional por nodo ───────────────────────────────────

#[utoipa::path(
    put,
    path = "/finanzas/egresos/{id}/distribuye",
    params(("id" = i32, Path, description = "Id del egreso a distribuir")),
    request_body = DistribuyeInput,
    responses(
        (status = 200, description = "Egreso distribuido correctamente", body = Value),
        (status = 400, description = "Distribución cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn distribuye_egreso(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<DistribuyeInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /finanzas/egresos/{}/distribuye nodo='{}'", id, body.nodo);

    let ret = svc::distribuye_egreso(&state.postgres, id, &body.nodo).await;

    if ret.afectado > 0 {
        info!("PUT /finanzas/egresos/{}/distribuye ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /finanzas/egresos/{}/distribuye ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Egresos por proveedor y proyecto ─────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/reportes/egresos-proveedor",
    params(
        ("tipo"      = bool,           Query, description = "false=agrupar por proveedor, true=por proyecto"),
        ("fecha_ini" = String,         Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String,         Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx"),
    ),
    responses(
        (status = 200, description = "Reporte egresos proveedor×proyecto", body = Value),
        (status = 400, description = "Fechas inválidas",                   body = Value),
        (status = 404, description = "Sin registros en el período",        body = Value),
        (status = 500, description = "Error de base de datos",             body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn egresos_proveedor_proyecto(
    State(state): State<AppState>,
    Query(q): Query<EgresosProveedorQuery>,
) -> Response {
    debug!("GET /finanzas/reportes/egresos-proveedor tipo={}", q.tipo);

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::egresos_proveedor_proyecto(&state.postgres, q.tipo, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /finanzas/reportes/egresos-proveedor ← 200 {} registros", lista.len());
            let total: rust_decimal::Decimal = lista.iter().filter_map(|e| e.monto).sum();
            let items: Vec<Value> = lista.iter().map(|e| json!({
                "tipo":      e.tipo,
                "proveedor": e.proveedor,
                "proyecto":  e.proyecto,
                "monto":     e.monto.map(|m| m.to_string()),
            })).collect();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::egresos_prov_proy(&items) {
                    Ok(b)  => render::xlsx_resp(b, "egresos_proveedor_proyecto.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "egresos": items, "total": total.to_string() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/reportes/egresos-proveedor ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /finanzas/reportes/egresos-proveedor ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Ingresos detalle por período ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/reportes/ingresos-detalle",
    params(
        ("fecha_ini" = String,         Query, description = "Inicio del período YYYY-MM-DD"),
        ("fecha_fin" = String,         Query, description = "Fin del período YYYY-MM-DD"),
        ("format"    = Option<String>, Query, description = "xlsx"),
    ),
    responses(
        (status = 200, description = "Detalle de ingresos del período", body = Value),
        (status = 400, description = "Fechas inválidas",                body = Value),
        (status = 404, description = "Sin registros en el período",     body = Value),
        (status = 500, description = "Error de base de datos",          body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn ingresos_detalle(
    State(state): State<AppState>,
    Query(q): Query<FechasQuery>,
) -> Response {
    debug!("GET /finanzas/reportes/ingresos-detalle");

    let fecha_ini = match parse_date(&q.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };
    let fecha_fin = match parse_date(&q.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))).into_response(),
    };

    match svc::ingresos_detalle(&state.postgres, fecha_ini, fecha_fin).await {
        Ok(lista) => {
            info!("GET /finanzas/reportes/ingresos-detalle ← 200 {} registros", lista.len());
            let total: rust_decimal::Decimal = lista.iter().filter_map(|i| i.monto).sum();
            let items: Vec<Value> = lista.iter().map(|i| json!({
                "id":           i.id,
                "fecha":        i.fecha.map(|d| d.to_string()),
                "banco":        i.banco,
                "cuenta":       i.cuenta,
                "forma_pago":   i.forma_pago,
                "proyecto":     i.proyecto,
                "monto":        i.monto.map(|m| m.to_string()),
                "referencia":   i.referencia,
                "comentario":   i.comentario,
                "fecha_aplica": i.fecha_aplica.map(|d| d.to_string()),
                "cliente":      i.cliente,
            })).collect();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::ingresos_detalle(&items) {
                    Ok(b)  => render::xlsx_resp(b, "ingresos_detalle.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "ingresos": items, "total": total.to_string() }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/reportes/ingresos-detalle ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /finanzas/reportes/ingresos-detalle ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Recibo de honorarios ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/recibo-honorarios/{id}",
    params(
        ("id"     = i32,           Path,  description = "Id del egreso o ingreso"),
        ("tipo"   = String,        Query, description = "Tipo de transacción: 'egreso' o 'ingreso'"),
        ("format" = Option<String>, Query, description = "pdf"),
    ),
    responses(
        (status = 200, description = "Datos del recibo de honorarios", body = Value),
        (status = 400, description = "Tipo inválido",                  body = Value),
        (status = 404, description = "Transacción no encontrada",      body = Value),
        (status = 500, description = "Error de base de datos",         body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn recibo_honorarios(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Query(q): Query<TipoQuery>,
) -> Response {
    debug!("GET /finanzas/recibo-honorarios/{} tipo={}", id, q.tipo);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };

    match svc::recibo_honorarios(&state.postgres, id, &q.tipo, tenant_id).await {
        Ok(r) => {
            info!("GET /finanzas/recibo-honorarios/{} ← 200 OK", id);
            let data = json!({
                "egr_ing":      r.egr_ing,
                "cte_prov":     r.cte_prov,
                "banco":        r.banco,
                "forma_pago":   r.forma_pago,
                "monto":        r.monto.to_string(),
                "fecha_aplica": r.fecha_aplica,
                "proyecto":     r.proyecto,
                "comentarios":  r.comentarios,
                "referencia":   r.referencia,
                "rfc_curp":     r.rfc_curp,
            });
            match q.format.as_deref() {
                Some("pdf") => match render::pdf::recibo_honorarios(&data) {
                    Ok(b)  => render::pdf_resp(b, &format!("recibo_honorarios_{}.pdf", id)),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(data)).into_response(),
            }
        }
        Err(rc) if rc.codigo == -1 && rc.mensaje.contains("inválido") => {
            (StatusCode::BAD_REQUEST, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/recibo-honorarios/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /finanzas/recibo-honorarios/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}
