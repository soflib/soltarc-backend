// Rutas:
//   POST   /finanzas/ingresos      → alta
//   DELETE /finanzas/ingresos/{id} → baja
//   PUT    /finanzas/ingresos      → cambios
//   GET    /finanzas/ingresos/{id} → consulta

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
    Json,
};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::ingresos::{Ingresos, IngresosFilter};
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::app_state::AppState;
use crate::services::finanzas::ingresos as svc;

#[derive(Debug, Deserialize)]
pub struct IngresosSearchQuery {
    pub proyecto:  Option<i32>,
    pub cliente:   Option<i32>,
    /// YYYY-MM-DD
    pub fecha_ini: Option<String>,
    /// YYYY-MM-DD
    pub fecha_fin: Option<String>,
    pub q:         Option<String>,
    pub page:      Option<i32>,
    pub size:      Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct IngresosLookupQuery {
    pub q:        Option<String>,
    pub proyecto: Option<i32>,
    pub cliente:  Option<i32>,
    pub limit:    Option<i32>,
}

fn parse_date_opt(s: Option<&str>, field: &str) -> Result<Option<NaiveDate>, String> {
    match s {
        None => Ok(None),
        Some(v) if v.is_empty() => Ok(None),
        Some(v) => NaiveDate::parse_from_str(v, "%Y-%m-%d")
            .map(Some)
            .map_err(|e| format!("{} inválida: {e}", field)),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct IngresosInput {
    pub id:          Option<i32>,
    pub banco:       i32,
    pub cuenta:      String,
    pub forma_pago:  String,
    pub proyecto:    i32,
    pub monto:       f64,
    pub referencia:  String,
    pub comentario:  String,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_aplica: String,
    pub cliente:     i32,
    /// UUID del usuario que registra (solo se usa en alta)
    pub usuario_ms:  String,
}

fn parse_input(body: IngresosInput) -> Result<Ingresos, String> {
    let fecha_aplica = NaiveDateTime::parse_from_str(&body.fecha_aplica, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("fecha_aplica inválida: {e}"))?;
    let monto = Decimal::try_from(body.monto)
        .map_err(|e| format!("monto inválido: {e}"))?;
    let usuario_ms = Uuid::parse_str(&body.usuario_ms)
        .map_err(|e| format!("usuario_ms UUID inválido: {e}"))?;

    Ok(Ingresos {
        id:              body.id,
        fecha:           chrono::Utc::now().naive_utc(),
        banco:           body.banco,
        banco_nombre:    None, // poblado por SP en lecturas
        cuenta:          body.cuenta,
        forma_pago:      body.forma_pago,
        proyecto:        body.proyecto,
        proyecto_nombre: None,
        monto,
        referencia:      body.referencia,
        comentario:      body.comentario,
        fecha_aplica,
        cliente:         body.cliente,
        cliente_nombre:  None,
        usuario_ms,
    })
}

fn ingreso_json(i: &Ingresos) -> Value {
    json!({
        "id":              i.id,
        "fecha":           i.fecha.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "banco":           i.banco,
        "banco_nombre":    i.banco_nombre,
        "cuenta":          i.cuenta,
        "forma_pago":      i.forma_pago,
        "proyecto":        i.proyecto,
        "proyecto_nombre": i.proyecto_nombre,
        "monto":           i.monto.to_string(),
        "referencia":      i.referencia,
        "comentario":      i.comentario,
        "fecha_aplica":    i.fecha_aplica.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "cliente":         i.cliente,
        "cliente_nombre":  i.cliente_nombre,
        "usuario_ms":      i.usuario_ms.to_string(),
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/finanzas/ingresos",
    request_body = IngresosInput,
    responses(
        (status = 201, description = "Ingreso registrado",       body = Value),
        (status = 400, description = "Alta cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<IngresosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = body.proyecto, monto = body.monto, "POST /finanzas/ingresos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ing = match parse_input(body) {
        Ok(i)    => i,
        Err(msg) => {
            error!("POST /finanzas/ingresos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &ing, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /finanzas/ingresos ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /finanzas/ingresos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/finanzas/ingresos/{id}",
    params(("id" = i32, Path, description = "Id del ingreso a eliminar")),
    responses(
        (status = 200, description = "Ingreso eliminado",        body = Value),
        (status = 400, description = "Baja cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /finanzas/ingresos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        info!("DELETE /finanzas/ingresos/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /finanzas/ingresos/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambios ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/finanzas/ingresos",
    request_body = IngresosInput,
    responses(
        (status = 200, description = "Ingreso actualizado",             body = Value),
        (status = 400, description = "Actualización cancelada o error", body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<IngresosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /finanzas/ingresos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let ing = match parse_input(body) {
        Ok(i)    => i,
        Err(msg) => {
            error!("PUT /finanzas/ingresos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambios(&state.postgres, &ing, tenant_id).await;

    if ret.afectado > 0 {
        info!("PUT /finanzas/ingresos ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /finanzas/ingresos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/ingresos/{id}",
    params(("id" = i32, Path, description = "Id del ingreso a consultar")),
    responses(
        (status = 200, description = "Ingreso encontrado",       body = Value),
        (status = 404, description = "Ingreso no encontrado",    body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /finanzas/ingresos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(i)) => {
            info!("GET /finanzas/ingresos/{} ← 200", id);
            (StatusCode::OK, Json(ingreso_json(&i)))
        }
        Ok(None) => {
            info!("GET /finanzas/ingresos/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Ingreso no encontrado" })))
        }
        Err(rc) => {
            error!("GET /finanzas/ingresos/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Listado paginado con filtros + texto libre ────────────────────────────────
//
// GET /finanzas/ingresos?proyecto=&cliente=&fecha_ini=&fecha_fin=&q=&page=1&size=25
//
// Response: { items: [...], total, page, size }

#[utoipa::path(
    get,
    path = "/finanzas/ingresos",
    params(
        ("proyecto"  = Option<i32>,    Query, description = "Filtra por proyecto"),
        ("cliente"   = Option<i32>,    Query, description = "Filtra por cliente"),
        ("fecha_ini" = Option<String>, Query, description = "Desde (YYYY-MM-DD)"),
        ("fecha_fin" = Option<String>, Query, description = "Hasta (YYYY-MM-DD)"),
        ("q"         = Option<String>, Query, description = "Texto libre (ILIKE referencia/comentario/cliente/proyecto)"),
        ("page"      = Option<i32>,    Query, description = "Página 1-based (default 1)"),
        ("size"      = Option<i32>,    Query, description = "Tamaño de página (default 25, máx 200)"),
    ),
    responses(
        (status = 200, description = "Listado paginado", body = Value),
        (status = 400, description = "Parámetro inválido", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn lista(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<IngresosSearchQuery>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let fecha_ini = match parse_date_opt(q.fecha_ini.as_deref(), "fecha_ini") {
        Ok(v)    => v,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))),
    };
    let fecha_fin = match parse_date_opt(q.fecha_fin.as_deref(), "fecha_fin") {
        Ok(v)    => v,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))),
    };

    let page = q.page.unwrap_or(1).max(1);
    let size = q.size.unwrap_or(25).clamp(1, 200);
    let offset = (page - 1) * size;

    let filtros = IngresosFilter {
        proyecto: q.proyecto,
        cliente:  q.cliente,
        fecha_ini,
        fecha_fin,
        q:        q.q,
        offset,
        limit:    size,
    };
    debug!("GET /finanzas/ingresos page={} size={} filtros={:?}", page, size, filtros);

    match svc::search(&state.postgres, &filtros, tenant_id).await {
        Ok(page_res) => {
            info!("GET /finanzas/ingresos ← 200 {}/{} items", page_res.items.len(), page_res.total);
            let items: Vec<Value> = page_res.items.iter().map(ingreso_json).collect();
            (StatusCode::OK, Json(json!({
                "items": items,
                "total": page_res.total,
                "page":  page_res.page,
                "size":  page_res.size,
            })))
        }
        Err(rc) => {
            error!("GET /finanzas/ingresos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Lookup (autocomplete) ─────────────────────────────────────────────────────
//
// GET /finanzas/ingresos/lookup?q=foo&proyecto=&cliente=&limit=20
// Etiqueta: "<fecha_aplica> · <cliente_nombre> · <referencia> ($monto)"

#[utoipa::path(
    get,
    path = "/finanzas/ingresos/lookup",
    params(
        ("q"        = Option<String>, Query, description = "Texto libre (ILIKE)"),
        ("proyecto" = Option<i32>,    Query, description = "Restringe a un proyecto"),
        ("cliente"  = Option<i32>,    Query, description = "Restringe a un cliente"),
        ("limit"    = Option<i32>,    Query, description = "Máximo (default 20, máx 100)"),
    ),
    responses(
        (status = 200, description = "Lista [{id, etiqueta}]", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn lookup(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<IngresosLookupQuery>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    let filtros = IngresosFilter {
        proyecto:  q.proyecto,
        cliente:   q.cliente,
        fecha_ini: None,
        fecha_fin: None,
        q:         q.q,
        offset:    0,
        limit,
    };
    debug!("GET /finanzas/ingresos/lookup filtros={:?}", filtros);

    match svc::search(&state.postgres, &filtros, tenant_id).await {
        Ok(page_res) => {
            let items: Vec<LookupItem> = page_res.items.into_iter().map(|i| LookupItem {
                id: i.id.unwrap_or(0),
                etiqueta: format!(
                    "{} · {} · {} (${})",
                    i.fecha_aplica.format("%Y-%m-%d"),
                    i.cliente_nombre.as_deref().unwrap_or(""),
                    i.referencia,
                    i.monto,
                ),
            }).collect();
            info!("GET /finanzas/ingresos/lookup ← 200 {} items", items.len());
            (StatusCode::OK, Json(json!(items)))
        }
        Err(rc) => {
            error!("GET /finanzas/ingresos/lookup ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
