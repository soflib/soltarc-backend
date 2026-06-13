// Rutas:
//   POST   /finanzas/saldos             → alta
//   DELETE /finanzas/saldos/{id}        → baja
//   PUT    /finanzas/saldos             → cambios
//   GET    /finanzas/saldos/{id}        → consulta
//   GET    /finanzas/saldos/banco/{id}  → saldos por banco
//   GET    /finanzas/saldos             → todos los saldos

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension,
    Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::saldos_bancos::SaldosBancos;
use crate::infrastructure::db::app_state::AppState;
use crate::services::finanzas::saldos_bancos as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaldosBancosInput {
    pub id:    Option<i32>,
    pub banco: i32,
    pub ano:   i32,
    pub mes:   i32,
    pub monto: f64,
}

fn parse_input(body: SaldosBancosInput) -> Result<SaldosBancos, String> {
    let monto = Decimal::try_from(body.monto)
        .map_err(|e| format!("monto inválido: {e}"))?;
    Ok(SaldosBancos {
        id:          body.id,
        banco:       body.banco,
        banco_nombre: None, // poblado por SP en lecturas
        ano:         body.ano,
        mes:         body.mes,
        monto,
    })
}

fn saldo_json(s: &SaldosBancos) -> Value {
    json!({
        "id":          s.id,
        "banco":       s.banco,
        "banco_nombre": s.banco_nombre,
        "ano":         s.ano,
        "mes":         s.mes,
        "monto":       s.monto.to_string(),
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/finanzas/saldos",
    request_body = SaldosBancosInput,
    responses(
        (status = 201, description = "Saldo registrado",         body = Value),
        (status = 400, description = "Alta cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<SaldosBancosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(banco = body.banco, ano = body.ano, mes = body.mes, "POST /finanzas/saldos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let sdo = match parse_input(body) {
        Ok(s)    => s,
        Err(msg) => {
            error!("POST /finanzas/saldos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &sdo, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /finanzas/saldos ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /finanzas/saldos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/finanzas/saldos/{id}",
    params(("id" = i32, Path, description = "Id del saldo a eliminar")),
    responses(
        (status = 200, description = "Saldo eliminado",          body = Value),
        (status = 400, description = "Baja cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /finanzas/saldos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        info!("DELETE /finanzas/saldos/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /finanzas/saldos/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambios ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/finanzas/saldos",
    request_body = SaldosBancosInput,
    responses(
        (status = 200, description = "Saldo actualizado",               body = Value),
        (status = 400, description = "Actualización cancelada o error", body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<SaldosBancosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /finanzas/saldos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let sdo = match parse_input(body) {
        Ok(s)    => s,
        Err(msg) => {
            error!("PUT /finanzas/saldos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambios(&state.postgres, &sdo, tenant_id).await;

    if ret.afectado > 0 {
        info!("PUT /finanzas/saldos ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /finanzas/saldos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/saldos/{id}",
    params(("id" = i32, Path, description = "Id del saldo a consultar")),
    responses(
        (status = 200, description = "Saldo encontrado",         body = Value),
        (status = 404, description = "Saldo no encontrado",      body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /finanzas/saldos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(s)) => {
            info!("GET /finanzas/saldos/{} ← 200", id);
            (StatusCode::OK, Json(saldo_json(&s)))
        }
        Ok(None) => {
            info!("GET /finanzas/saldos/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Saldo no encontrado" })))
        }
        Err(rc) => {
            error!("GET /finanzas/saldos/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Saldos por banco ──────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/saldos/banco/{id}",
    params(("id" = i32, Path, description = "Id del banco (catálogo tipo 5)")),
    responses(
        (status = 200, description = "Saldos del banco",         body = Value),
        (status = 404, description = "Sin registros",            body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn saldos_x_banco(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(banco): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!(banco, "GET /finanzas/saldos/banco/{}", banco);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::saldos_x_banco(&state.postgres, banco, tenant_id).await {
        Ok(lista) => {
            info!("GET /finanzas/saldos/banco/{} ← 200 {} registros", banco, lista.len());
            let items: Vec<Value> = lista.iter().map(saldo_json).collect();
            (StatusCode::OK, Json(json!({ "saldos": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/saldos/banco/{} ← 404", banco);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /finanzas/saldos/banco/{} ← 500 codigo={}", banco, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Todos los saldos ──────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/saldos",
    responses(
        (status = 200, description = "Todos los saldos",         body = Value),
        (status = 404, description = "Sin registros",            body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn saldos_todos(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /finanzas/saldos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::saldos_todos(&state.postgres, tenant_id).await {
        Ok(lista) => {
            info!("GET /finanzas/saldos ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(saldo_json).collect();
            (StatusCode::OK, Json(json!({ "saldos": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/saldos ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /finanzas/saldos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
