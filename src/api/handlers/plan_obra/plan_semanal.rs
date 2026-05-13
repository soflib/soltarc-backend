// Programa...: handler::plan_obra::plan_semanal
// Descripción: Endpoints HTTP para Plan Semanal
// Origen.....: oPlanSemanal.cs
//
// Rutas:
//   GET /plan-semanal/fechas?proyecto=
//   GET /plan-semanal/partidas?proyecto=&fecha_ini=&nivel=

use axum::{extract::{Query, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use time::macros::format_description;
use tracing::{debug, error, info};

use crate::domain::models::plan_semanal::PartidasSemanal;
use crate::infrastructure::db::app_state::AppState;
use crate::services::plan_obra::plan_semanal as svc;

// ─────────────────────────────────────────────
// QUERY PARAMS
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct FiltroPlanSem {
    pub proyecto:  Option<i32>,
    pub fecha_ini: Option<String>,
    pub nivel:     Option<i32>,
}

// ─────────────────────────────────────────────
// HELPER
// ─────────────────────────────────────────────

fn partida_json(p: &PartidasSemanal) -> Value {
    let fmt = format_description!("[year]-[month]-[day]");
    json!({
        "nodo":        p.nodo,
        "nivel":       p.nivel,
        "descripcion": p.descripcion,
        "fecha_inicio": p.fecha_inicio.map(|d| d.format(&fmt).unwrap_or_default()),
        "fecha_fin":    p.fecha_fin.map(|d| d.format(&fmt).unwrap_or_default()),
        "cuando_ini":  p.cuando_ini,
        "cuando_fin":  p.cuando_fin,
        "estado":      p.estado,
    })
}

// ─────────────────────────────────────────────
// FECHAS — GET /plan-semanal/fechas?proyecto=
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/plan-semanal/fechas",
    params(("proyecto" = i32, Query, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Rango de fechas y número de semanas", body = Value),
        (status = 400, description = "Falta parámetro proyecto",            body = Value),
        (status = 404, description = "Sin datos para el proyecto",          body = Value),
        (status = 500, description = "Error de base de datos",              body = Value),
    ),
    tag = "PlanSemanal"
)]
pub async fn fechas(
    State(state): State<AppState>,
    Query(q): Query<FiltroPlanSem>,
) -> (StatusCode, Json<Value>) {
    let proyecto = match q.proyecto {
        Some(id) => id,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Falta parámetro: proyecto" })));
        }
    };
    debug!("GET /plan-semanal/fechas?proyecto={}", proyecto);

    let fmt = format_description!("[year]-[month]-[day]");

    match svc::fechas(&state.postgres, proyecto).await {
        Ok(f) => {
            info!("GET /plan-semanal/fechas ← 200 proyecto={}", proyecto);
            (StatusCode::OK, Json(json!({
                "proyecto":    proyecto,
                "fecha_ini":   f.fecha_ini.format(&fmt).unwrap_or_default(),
                "fecha_fin":   f.fecha_fin.format(&fmt).unwrap_or_default(),
                "num_semanas": f.num_semanas,
            })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /plan-semanal/fechas ← 500 {}", ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /plan-semanal/fechas ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CARGA PARTIDAS — GET /plan-semanal/partidas?proyecto=&fecha_ini=&nivel=
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/plan-semanal/partidas",
    params(
        ("proyecto"  = i32,    Query, description = "Id del proyecto"),
        ("fecha_ini" = String, Query, description = "Fecha inicio del plan YYYY-MM-DD"),
        ("nivel"     = i32,    Query, description = "Nivel jerárquico a mostrar"),
    ),
    responses(
        (status = 200, description = "Partidas del plan semanal",  body = Value),
        (status = 400, description = "Faltan parámetros",          body = Value),
        (status = 404, description = "Sin partidas",               body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "PlanSemanal"
)]
pub async fn carga_partidas(
    State(state): State<AppState>,
    Query(q): Query<FiltroPlanSem>,
) -> (StatusCode, Json<Value>) {
    let proyecto = match q.proyecto {
        Some(id) => id,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Falta parámetro: proyecto" })));
        }
    };
    let fecha_str = match q.fecha_ini {
        Some(ref s) => s.clone(),
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Falta parámetro: fecha_ini" })));
        }
    };
    let nivel = q.nivel.unwrap_or(2);

    let fmt = format_description!("[year]-[month]-[day]");
    let fecha_ini = match time::Date::parse(&fecha_str, &fmt) {
        Ok(d) => d,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "fecha_ini inválida, use YYYY-MM-DD" })));
        }
    };
    debug!("GET /plan-semanal/partidas?proyecto={}&nivel={}", proyecto, nivel);

    match svc::carga_partidas(&state.postgres, proyecto, fecha_ini, nivel).await {
        Ok(lista) => {
            info!("GET /plan-semanal/partidas ← 200 {} partidas", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(partida_json).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /plan-semanal/partidas ← 500 {}", ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /plan-semanal/partidas ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
