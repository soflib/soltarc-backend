// Rutas:
//   POST   /finanzas/egresos            → alta
//   DELETE /finanzas/egresos/{id}       → baja
//   PUT    /finanzas/egresos            → cambios
//   GET    /finanzas/egresos/{id}       → consulta
//   GET    /finanzas/egresos?proyecto=  → lista por proyecto

use axum::{
    extract::{Path, Query, State},
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

use crate::domain::models::egresos::Egresos;
use crate::infrastructure::db::app_state::AppState;
use crate::services::finanzas::egresos as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct EgresosInput {
    pub id:           Option<i32>,
    pub banco:        i32,
    pub cuenta:       String,
    pub forma_pago:   String,
    pub centro_costo: i32,
    pub monto:        f64,
    pub referencia:   String,
    pub comentario:   String,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_aplica: String,
    pub proyecto:     i32,
    pub proveedor:    i32,
    /// UUID del usuario que registra
    pub usuario:      String,
}

#[derive(Debug, Deserialize)]
pub struct EgresosQuery {
    pub proyecto: Option<i32>,
}

fn parse_input(body: EgresosInput) -> Result<Egresos, String> {
    let fecha_aplica = NaiveDateTime::parse_from_str(&body.fecha_aplica, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("fecha_aplica inválida: {e}"))?;
    let monto = Decimal::try_from(body.monto)
        .map_err(|e| format!("monto inválido: {e}"))?;
    let usuario = Uuid::parse_str(&body.usuario)
        .map_err(|e| format!("usuario UUID inválido: {e}"))?;

    Ok(Egresos {
        id:                    body.id,
        fecha:                 chrono::Utc::now().naive_utc(),
        banco:                 body.banco,
        banco_nombre:          None, // poblado por SP en lecturas
        cuenta:                body.cuenta,
        forma_pago:            body.forma_pago,
        centro_costo:          body.centro_costo,
        centro_costo_nombre:   None,
        monto,
        referencia:            body.referencia,
        comentario:            body.comentario,
        fecha_aplica,
        proyecto:              body.proyecto,
        proyecto_nombre:       None,
        proveedor:             body.proveedor,
        proveedor_nombre:      None,
        usuario,
    })
}

fn egreso_json(e: &Egresos) -> Value {
    json!({
        "id":                   e.id,
        "fecha":                e.fecha.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "banco":                e.banco,
        "banco_nombre":         e.banco_nombre,
        "cuenta":               e.cuenta,
        "forma_pago":           e.forma_pago,
        "centro_costo":         e.centro_costo,
        "centro_costo_nombre":  e.centro_costo_nombre,
        "monto":                e.monto.to_string(),
        "referencia":           e.referencia,
        "comentario":           e.comentario,
        "fecha_aplica":         e.fecha_aplica.format("%Y-%m-%dT%H:%M:%S").to_string(),
        "proyecto":             e.proyecto,
        "proyecto_nombre":      e.proyecto_nombre,
        "proveedor":            e.proveedor,
        "proveedor_nombre":     e.proveedor_nombre,
        "usuario":              e.usuario.to_string(),
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/finanzas/egresos",
    request_body = EgresosInput,
    responses(
        (status = 201, description = "Egreso registrado",        body = Value),
        (status = 400, description = "Alta cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<EgresosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = body.proyecto, monto = body.monto, "POST /finanzas/egresos");

    let egr = match parse_input(body) {
        Ok(e)    => e,
        Err(msg) => {
            error!("POST /finanzas/egresos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &egr).await;

    if ret.afectado > 0 {
        info!("POST /finanzas/egresos ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /finanzas/egresos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/finanzas/egresos/{id}",
    params(("id" = i32, Path, description = "Id del egreso a eliminar")),
    responses(
        (status = 200, description = "Egreso eliminado",         body = Value),
        (status = 400, description = "Baja cancelada o error",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /finanzas/egresos/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /finanzas/egresos/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /finanzas/egresos/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambios ───────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/finanzas/egresos",
    request_body = EgresosInput,
    responses(
        (status = 200, description = "Egreso actualizado",              body = Value),
        (status = 400, description = "Actualización cancelada o error", body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<EgresosInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /finanzas/egresos");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })));
    };

    let egr = match parse_input(body) {
        Ok(e)    => e,
        Err(msg) => {
            error!("PUT /finanzas/egresos ← 400 parse error: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambios(&state.postgres, &egr).await;

    if ret.afectado > 0 {
        info!("PUT /finanzas/egresos ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /finanzas/egresos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/egresos/{id}",
    params(("id" = i32, Path, description = "Id del egreso a consultar")),
    responses(
        (status = 200, description = "Egreso encontrado",        body = Value),
        (status = 404, description = "Egreso no encontrado",     body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /finanzas/egresos/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(e)) => {
            info!("GET /finanzas/egresos/{} ← 200", id);
            (StatusCode::OK, Json(egreso_json(&e)))
        }
        Ok(None) => {
            info!("GET /finanzas/egresos/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Egreso no encontrado" })))
        }
        Err(rc) => {
            error!("GET /finanzas/egresos/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Total egresos por proyecto ────────────────────────────────────────────────
// Origen C#: oEgresos.TotalEgresos — usado en EncaProyecto.ascx.cs (header admin)
// y en ProyectosDetalleTareasM.aspx.cs para mostrar gasto total del proyecto.

#[utoipa::path(
    get,
    path = "/finanzas/egresos/total",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Total de egresos del proyecto", body = Value),
        (status = 400, description = "Falta parámetro proyecto",      body = Value),
        (status = 500, description = "Error de base de datos",        body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn total(
    State(state): State<AppState>,
    Query(q): Query<EgresosQuery>,
) -> (StatusCode, Json<Value>) {
    let Some(proyecto) = q.proyecto else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El parámetro 'proyecto' es requerido" })));
    };

    debug!(proyecto, "GET /finanzas/egresos/total");

    match svc::total_egresos(&state.postgres, proyecto).await {
        Ok(total) => {
            info!("GET /finanzas/egresos/total?proyecto={} ← 200 total={}", proyecto, total);
            (StatusCode::OK, Json(json!({ "proyecto": proyecto, "total_egresos": total.to_string() })))
        }
        Err(rc) => {
            error!("GET /finanzas/egresos/total?proyecto={} ← 500 codigo={}", proyecto, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Lista por proyecto ────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/finanzas/egresos",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto a consultar"),
    ),
    responses(
        (status = 200, description = "Lista de egresos",         body = Value),
        (status = 400, description = "Falta parámetro proyecto", body = Value),
        (status = 404, description = "Sin egresos",              body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Finanzas"
)]
pub async fn lista(
    State(state): State<AppState>,
    Query(q): Query<EgresosQuery>,
) -> (StatusCode, Json<Value>) {
    let Some(proyecto) = q.proyecto else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El parámetro 'proyecto' es requerido" })));
    };

    debug!(proyecto, "GET /finanzas/egresos");

    match svc::carga_egresos_proy_xref(&state.postgres, proyecto).await {
        Ok(lista) => {
            info!("GET /finanzas/egresos?proyecto={} ← 200 {} registros", proyecto, lista.len());
            let items: Vec<Value> = lista.iter().map(egreso_json).collect();
            (StatusCode::OK, Json(json!({ "egresos": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -50 => {
            info!("GET /finanzas/egresos?proyecto={} ← 404", proyecto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /finanzas/egresos?proyecto={} ← 500 codigo={}", proyecto, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
