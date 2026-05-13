// Programa...: handler::ppto::ppto_a_proyecto
// Origen.....: oPPTOaProyecto.cs
//
// Rutas:
//   GET  /ppto/a-proyecto/num-partidas?proyecto=
//   GET  /ppto/a-proyecto/nodos?ppto=&proyecto=
//   POST /ppto/a-proyecto
//   GET  /ppto/a-proyecto/tipo-proyecto?proyecto=

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use time::macros::format_description;
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::infrastructure::db::app_state::AppState;
use crate::services::ppto::ppto_a_proyecto as svc;

fn parse_date(s: &str) -> Result<time::Date, String> {
    let fmt = format_description!("[year]-[month]-[day]");
    time::Date::parse(s, &fmt).map_err(|e| format!("fecha inválida '{}': {}", s, e))
}

#[derive(Debug, Deserialize)]
pub struct NumPartidasQuery {
    pub proyecto: i32,
}

#[derive(Debug, Deserialize)]
pub struct CargaNodosQuery {
    pub ppto:    i32,
    pub proyecto: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreaPptoProyInput {
    pub ppto:      i32,
    pub proyecto:  i32,
    pub fecha_ini: String,
    pub fecha_fin: String,
    pub estado:    i32,
    pub tipo:      i32,
}

#[derive(Debug, Deserialize)]
pub struct TipoProyectoQuery {
    pub proyecto: i32,
}

// ── Consulta número de partidas ───────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/a-proyecto/num-partidas",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Número de partidas en el proyecto", body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn consulta_numero_partidas(
    State(state): State<AppState>,
    Query(q): Query<NumPartidasQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = q.proyecto, "GET /ppto/a-proyecto/num-partidas");

    let ret = svc::consulta_numero_partidas(&state.postgres, q.proyecto).await;

    info!("GET /ppto/a-proyecto/num-partidas?proyecto={} ← codigo={} afectado={}", q.proyecto, ret.codigo, ret.afectado);
    (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
}

// ── Carga nodos del presupuesto para un proyecto ──────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/a-proyecto/nodos",
    params(
        ("ppto"     = i32, Query, description = "Id del presupuesto"),
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Nodos/partidas del presupuesto",  body = Value),
        (status = 404, description = "Sin nodos encontrados",           body = Value),
        (status = 500, description = "Error de base de datos",          body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn carga_nodos(
    State(state): State<AppState>,
    Query(q): Query<CargaNodosQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(ppto = q.ppto, proyecto = q.proyecto, "GET /ppto/a-proyecto/nodos");

    match svc::carga_nodos(&state.postgres, q.ppto, q.proyecto).await {
        Ok(lista) => {
            info!("GET /ppto/a-proyecto/nodos ← 200 {} nodos", lista.len());
            let items: Vec<Value> = lista.iter().map(|n| json!({
                "id":          n.id,
                "nodo":        n.nodo,
                "concepto":    n.concepto,
                "nivel":       n.nivel,
                "unidad":      n.unidad,
                "cantidad":    n.cantidad.map(|d| d.to_string()),
                "precio_u":    n.precio_u.map(|d| d.to_string()),
                "calculo":     n.calculo,
                "presupuesto": n.presupuesto,
            })).collect();
            (StatusCode::OK, Json(json!({ "nodos": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -75 => {
            info!("GET /ppto/a-proyecto/nodos ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/a-proyecto/nodos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Crea partidas del proyecto desde presupuesto ──────────────────────────────

#[utoipa::path(
    post,
    path = "/ppto/a-proyecto",
    request_body = CreaPptoProyInput,
    responses(
        (status = 201, description = "Partidas creadas exitosamente", body = Value),
        (status = 400, description = "Operación cancelada o error",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn crea_partidas_proyecto(
    State(state): State<AppState>,
    Json(body): Json<CreaPptoProyInput>,
) -> (StatusCode, Json<Value>) {
    debug!(ppto = body.ppto, proyecto = body.proyecto, "POST /ppto/a-proyecto");

    let fecha_ini = match parse_date(&body.fecha_ini) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))),
    };
    let fecha_fin = match parse_date(&body.fecha_fin) {
        Ok(d)    => d,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg }))),
    };

    let ret = svc::crea_partidas_proyecto(
        &state.postgres,
        body.ppto,
        body.proyecto,
        fecha_ini,
        fecha_fin,
        body.estado,
        body.tipo,
    ).await;

    if ret.afectado > 0 {
        info!("POST /ppto/a-proyecto ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /ppto/a-proyecto ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Obtiene tipo de proyecto ──────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/a-proyecto/tipo-proyecto",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Tipo del proyecto",     body = Value),
        (status = 404, description = "Proyecto no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn obtiene_tipo_proyecto(
    State(state): State<AppState>,
    Query(q): Query<TipoProyectoQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = q.proyecto, "GET /ppto/a-proyecto/tipo-proyecto");

    match svc::obtiene_tipo_proyecto(&state.postgres, q.proyecto).await {
        Ok(tipo) => {
            info!("GET /ppto/a-proyecto/tipo-proyecto?proyecto={} ← 200 tipo={}", q.proyecto, tipo);
            (StatusCode::OK, Json(json!({ "tipo": tipo })))
        }
        Err(rc) if rc.codigo > -75 => {
            info!("GET /ppto/a-proyecto/tipo-proyecto?proyecto={} ← 404", q.proyecto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/a-proyecto/tipo-proyecto ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
