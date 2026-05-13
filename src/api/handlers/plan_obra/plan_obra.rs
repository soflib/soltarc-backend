// Programa...: handler::plan_obra::plan_obra
// Descripción: Endpoints HTTP para Plan de Obra
// Origen.....: oPlanObra.cs
//
// Rutas:
//   PUT  /plan-obra/partidas                     → partida_upd_fecha
//   GET  /plan-obra/partidas?proyecto=           → partida_proyecto
//   GET  /plan-obra/avance?proyecto=&nivel=      → obtiene_avance
//   GET  /plan-obra/existe-plan?proyecto=        → existe_plan
//   POST /plan-obra/crea-plan                    → crea_plan
//   GET  /plan-obra/descendientes?nodo=          → descendientes_nodo

use axum::{extract::{Query, State}, http::StatusCode, Json};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::{json, Value};
use time::macros::format_description;
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::plan_obra::PlanObra;
use crate::infrastructure::db::app_state::AppState;
use crate::services::plan_obra::plan_obra as svc;

// ─────────────────────────────────────────────
// INPUT STRUCTS
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlanObraInput {
    pub id:           i32,
    /// Formato: "YYYY-MM-DD"
    pub fecha_ini:    String,
    /// Formato: "YYYY-MM-DD"
    pub fecha_fin:    String,
    pub estado:       i32,
    /// Formato: "YYYY-MM-DD"
    pub fecha_termina: String,
}

#[derive(Debug, Deserialize)]
pub struct FiltroProyecto {
    pub proyecto: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FiltroAvance {
    pub proyecto: Option<i32>,
    pub nivel:    Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FiltroNodo {
    pub nodo: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreaPlanInput {
    pub proyecto:  i32,
    /// Formato: "YYYY-MM-DD"
    pub fecha_ini: String,
    /// Formato: "YYYY-MM-DD"
    pub fecha_fin: String,
    pub estado:    i32,
}

// ─────────────────────────────────────────────
// HELPER
// ─────────────────────────────────────────────

fn parse_input(body: PlanObraInput) -> Result<PlanObra, String> {
    let parse_dt = |s: &str, campo: &str| {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|e| format!("{campo} inválida: {e}"))
            .and_then(|d| d.and_hms_opt(0, 0, 0).ok_or_else(|| format!("{campo}: hora inválida")))
    };

    Ok(PlanObra {
        id:           body.id,
        fecha_ini:    parse_dt(&body.fecha_ini,    "fecha_ini")?,
        fecha_fin:    parse_dt(&body.fecha_fin,    "fecha_fin")?,
        estado:       body.estado,
        comentarios:  String::new(),
        fecha_termina: parse_dt(&body.fecha_termina, "fecha_termina")?,
    })
}

fn plan_obra_json(p: &PlanObra) -> Value {
    json!({
        "id":            p.id,
        "fecha_ini":     p.fecha_ini.format("%Y-%m-%d").to_string(),
        "fecha_fin":     p.fecha_fin.format("%Y-%m-%d").to_string(),
        "estado":        p.estado,
        "comentarios":   p.comentarios,
        "fecha_termina": p.fecha_termina.format("%Y-%m-%d").to_string(),
    })
}

// ─────────────────────────────────────────────
// PARTIDA UPD FECHA — PUT /plan-obra/partidas
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/plan-obra/partidas",
    request_body = PlanObraInput,
    responses(
        (status = 200, description = "Fecha/estado actualizados",       body = Value),
        (status = 400, description = "Actualización cancelada o error", body = Value),
    ),
    tag = "PlanObra"
)]
pub async fn partida_upd_fecha(
    State(state): State<AppState>,
    Json(body): Json<PlanObraInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /plan-obra/partidas → id={}", body.id);

    let pla = match parse_input(body) {
        Ok(p) => p,
        Err(msg) => {
            error!("PUT /plan-obra/partidas ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::partida_upd_fecha(&state.postgres, &pla).await;

    if ret.afectado > 0 {
        info!("PUT /plan-obra/partidas ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /plan-obra/partidas ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// PARTIDAS PROYECTO — GET /plan-obra/partidas?proyecto=
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/plan-obra/partidas",
    params(("proyecto" = i32, Query, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Lista de partidas del plan",  body = Value),
        (status = 400, description = "Falta parámetro proyecto",    body = Value),
        (status = 404, description = "Sin partidas",                body = Value),
        (status = 500, description = "Error de base de datos",      body = Value),
    ),
    tag = "PlanObra"
)]
pub async fn partida_proyecto(
    State(state): State<AppState>,
    Query(q): Query<FiltroProyecto>,
) -> (StatusCode, Json<Value>) {
    let proyecto = match q.proyecto {
        Some(id) => id,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Falta parámetro: proyecto" })));
        }
    };
    debug!("GET /plan-obra/partidas?proyecto={}", proyecto);

    match svc::partida_proyecto(&state.postgres, proyecto).await {
        Ok(lista) => {
            info!("GET /plan-obra/partidas ← 200 {} partidas", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(plan_obra_json).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /plan-obra/partidas ← 500 {}", ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /plan-obra/partidas ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE AVANCE — GET /plan-obra/avance?proyecto=&nivel=
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/plan-obra/avance",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
        ("nivel"    = i32, Query, description = "Nivel jerárquico (1 = raíz)"),
    ),
    responses(
        (status = 200, description = "Avance del plan",            body = Value),
        (status = 400, description = "Faltan parámetros",          body = Value),
        (status = 404, description = "Sin datos",                  body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "PlanObra"
)]
pub async fn obtiene_avance(
    State(state): State<AppState>,
    Query(q): Query<FiltroAvance>,
) -> (StatusCode, Json<Value>) {
    let (proyecto, nivel) = match (q.proyecto, q.nivel) {
        (Some(p), Some(n)) => (p, n),
        _ => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Faltan parámetros: proyecto y nivel" })));
        }
    };
    debug!("GET /plan-obra/avance?proyecto={}&nivel={}", proyecto, nivel);

    match svc::obtiene_avance(&state.postgres, proyecto, nivel).await {
        Ok(lista) => {
            info!("GET /plan-obra/avance ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(plan_obra_json).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /plan-obra/avance ← 500 {}", ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /plan-obra/avance ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// EXISTE PLAN — GET /plan-obra/existe-plan?proyecto=
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/plan-obra/existe-plan",
    params(("proyecto" = i32, Query, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Conteo de partidas con y sin plan de fechas", body = Value),
        (status = 400, description = "Falta parámetro proyecto",                    body = Value),
        (status = 404, description = "Proyecto no encontrado",                      body = Value),
        (status = 500, description = "Error de base de datos",                      body = Value),
    ),
    tag = "PlanObra"
)]
pub async fn existe_plan(
    State(state): State<AppState>,
    Query(q): Query<FiltroProyecto>,
) -> (StatusCode, Json<Value>) {
    let proyecto = match q.proyecto {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Falta parámetro: proyecto" }))),
    };
    debug!("GET /plan-obra/existe-plan?proyecto={}", proyecto);

    match svc::existe_plan(&state.postgres, proyecto).await {
        Ok(s) => {
            info!("GET /plan-obra/existe-plan ← 200 total={} con_fecha={}", s.total_partidas, s.con_fecha);
            (StatusCode::OK, Json(json!({
                "proyecto":       proyecto,
                "total_partidas": s.total_partidas,
                "con_fecha":      s.con_fecha,
                "pendientes":     s.total_partidas - s.con_fecha,
                "plan_completo":  s.total_partidas == s.con_fecha,
            })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /plan-obra/existe-plan ← 500 {}", ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CREA PLAN — POST /plan-obra/crea-plan
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/plan-obra/crea-plan",
    request_body = CreaPlanInput,
    responses(
        (status = 200, description = "Plan de fechas creado",          body = Value),
        (status = 400, description = "Error al crear o fecha inválida", body = Value),
    ),
    tag = "PlanObra"
)]
pub async fn crea_plan(
    State(state): State<AppState>,
    Json(body): Json<CreaPlanInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /plan-obra/crea-plan → proyecto={}", body.proyecto);

    let fmt = format_description!("[year]-[month]-[day]");
    let fecha_ini = match time::Date::parse(&body.fecha_ini, &fmt) {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "fecha_ini inválida, use YYYY-MM-DD" }))),
    };
    let fecha_fin = match time::Date::parse(&body.fecha_fin, &fmt) {
        Ok(d) => d,
        Err(_) => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "fecha_fin inválida, use YYYY-MM-DD" }))),
    };

    let ret = svc::crea_plan(&state.postgres, body.proyecto, fecha_ini, fecha_fin, body.estado).await;

    if ret.afectado > 0 {
        info!("POST /plan-obra/crea-plan ← 200 creados={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /plan-obra/crea-plan ← 400 {}", ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// DESCENDIENTES NODO — GET /plan-obra/descendientes?nodo=
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/plan-obra/descendientes",
    params(("nodo" = String, Query, description = "Nodo raíz (ej: /42/1/3/)")),
    responses(
        (status = 200, description = "Partidas descendientes del nodo", body = Value),
        (status = 400, description = "Falta parámetro nodo",            body = Value),
        (status = 404, description = "Sin descendientes",               body = Value),
        (status = 500, description = "Error de base de datos",          body = Value),
    ),
    tag = "PlanObra"
)]
pub async fn descendientes_nodo(
    State(state): State<AppState>,
    Query(q): Query<FiltroNodo>,
) -> (StatusCode, Json<Value>) {
    let nodo = match q.nodo {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Falta parámetro: nodo" }))),
    };
    debug!("GET /plan-obra/descendientes?nodo={}", nodo);

    match svc::descendientes_nodo(&state.postgres, &nodo).await {
        Ok(lista) => {
            info!("GET /plan-obra/descendientes ← 200 {} nodos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(plan_obra_json).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /plan-obra/descendientes ← 500 {}", ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
