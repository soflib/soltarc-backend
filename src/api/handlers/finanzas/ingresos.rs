// Rutas:
//   POST   /finanzas/ingresos      → alta
//   DELETE /finanzas/ingresos/{id} → baja
//   PUT    /finanzas/ingresos      → cambios
//   GET    /finanzas/ingresos/{id} → consulta

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use uuid::Uuid;
use tracing::{debug, error, info};

use crate::domain::models::ingresos::Ingresos;
use crate::infrastructure::db::app_state::AppState;
use crate::services::finanzas::ingresos as svc;

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
    Json(body): Json<IngresosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = body.proyecto, monto = body.monto, "POST /finanzas/ingresos");

    let ing = match parse_input(body) {
        Ok(i)    => i,
        Err(msg) => {
            error!("POST /finanzas/ingresos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &ing).await;

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /finanzas/ingresos/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

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
    Json(body): Json<IngresosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /finanzas/ingresos");

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

    let ret = svc::cambios(&state.postgres, &ing).await;

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
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /finanzas/ingresos/{}", id);

    match svc::consulta(&state.postgres, id).await {
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
