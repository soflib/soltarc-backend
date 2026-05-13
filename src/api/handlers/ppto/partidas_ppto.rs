// Rutas:
//   POST /ppto/partidas                      → alta
//   PUT  /ppto/partidas                      → cambio
//   GET  /ppto/partidas?presupuesto=         → carga_partidas
//   DELETE /ppto/partidas/{id}?nodo=         → borra
//   PUT  /ppto/partidas/{id}/nodo            → actualiza_nodo
//   GET  /ppto/partidas/nuevo-nodo           → nuevo_nodo (?ppto=&nodo=&nivel=)
//   GET  /ppto/partidas/nivel2               → carga_2_nivel (?nodo=&ppto=)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::partidas_ppto::PartidasPpto;
use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::render;
use crate::services::ppto::partidas_ppto as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PartidasPptoInput {
    pub id:          Option<i32>,
    pub presupuesto: Option<i32>,
    pub nodo:        Option<String>,
    pub concepto:    Option<String>,
    pub unidad:      Option<i32>,
    pub cantidad:    Option<f64>,
    pub precio_u:    Option<f64>,
    pub calculo:     Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizaNodoInput {
    pub ppto:       i32,
    pub nuevo_nodo: String,
}

#[derive(Debug, Deserialize)]
pub struct PartidasQuery {
    pub presupuesto: Option<i32>,
    pub format:      Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BorraQuery {
    pub nodo: String,
}

#[derive(Debug, Deserialize)]
pub struct NuevoNodoQuery {
    pub ppto:  i32,
    pub nodo:  String,
    pub nivel: i32,
}

#[derive(Debug, Deserialize)]
pub struct Carga2NivelQuery {
    pub nodo: i32,
    pub ppto: i32,
}

fn parse_input(body: PartidasPptoInput) -> Result<PartidasPpto, String> {
    let cantidad = body.cantidad
        .map(|c| Decimal::try_from(c).map_err(|e| format!("cantidad inválida: {e}")))
        .transpose()?;
    let precio_u = body.precio_u
        .map(|p| Decimal::try_from(p).map_err(|e| format!("precio_u inválido: {e}")))
        .transpose()?;

    Ok(PartidasPpto {
        id:                 body.id,
        presupuesto:        body.presupuesto,
        nodo:               body.nodo,
        concepto:           body.concepto,
        unidad_nombre:      None,
        unidad:             body.unidad,
        cantidad,
        precio_u,
        importe_calculado:  None,
        calculo:            body.calculo,
        nivel:              None,
    })
}

fn partida_json(p: &PartidasPpto) -> Value {
    json!({
        "id":                 p.id,
        "presupuesto":        p.presupuesto,
        "nodo":               p.nodo,
        "concepto":           p.concepto,
        "unidad_nombre":      p.unidad_nombre,
        "unidad":             p.unidad,
        "cantidad":           p.cantidad.map(|d| d.to_string()),
        "precio_u":           p.precio_u.map(|d| d.to_string()),
        "importe_calculado":  p.importe_calculado.map(|d| d.to_string()),
        "calculo":            p.calculo,
        "nivel":              p.nivel,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/ppto/partidas",
    request_body = PartidasPptoInput,
    responses(
        (status = 201, description = "Partida registrada",      body = Value),
        (status = 400, description = "Alta cancelada o error",  body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<PartidasPptoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(presupuesto = ?body.presupuesto, nodo = ?body.nodo, "POST /ppto/partidas");

    let par = match parse_input(body) {
        Ok(p)    => p,
        Err(msg) => {
            error!("POST /ppto/partidas ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &par).await;

    if ret.afectado > 0 {
        info!("POST /ppto/partidas ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /ppto/partidas ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Borra ─────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/ppto/partidas/{id}",
    params(
        ("id"   = i32,    Path,  description = "Id de la partida a eliminar"),
        ("nodo" = String, Query, description = "Nodo de la partida"),
    ),
    responses(
        (status = 200, description = "Partida eliminada",              body = Value),
        (status = 400, description = "Baja cancelada (tiene hijos)",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn borra(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Query(q): Query<BorraQuery>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /ppto/partidas/{} nodo={}", id, q.nodo);

    let ret = svc::borra(&state.postgres, id, &q.nodo).await;

    if ret.afectado > 0 {
        info!("DELETE /ppto/partidas/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /ppto/partidas/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambio ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/ppto/partidas",
    request_body = PartidasPptoInput,
    responses(
        (status = 200, description = "Partida actualizada",                  body = Value),
        (status = 400, description = "Actualización cancelada o error",      body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Json(body): Json<PartidasPptoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /ppto/partidas");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambio" })));
    };

    let par = match parse_input(body) {
        Ok(p)    => p,
        Err(msg) => {
            error!("PUT /ppto/partidas ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambio(&state.postgres, &par).await;

    if ret.afectado > 0 {
        info!("PUT /ppto/partidas ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /ppto/partidas ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Carga Partidas ────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/partidas",
    params(
        ("presupuesto" = i32,           Query, description = "Id del presupuesto"),
        ("format"      = Option<String>, Query, description = "xlsx | pdf"),
    ),
    responses(
        (status = 200, description = "Lista de partidas",       body = Value),
        (status = 400, description = "Falta parámetro",         body = Value),
        (status = 404, description = "Sin partidas",            body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn carga_partidas(
    State(state): State<AppState>,
    Query(q): Query<PartidasQuery>,
) -> Response {
    let Some(presupuesto) = q.presupuesto else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Se requiere el parámetro 'presupuesto'" }))).into_response();
    };

    debug!(presupuesto, "GET /ppto/partidas");

    match svc::carga_partidas(&state.postgres, presupuesto).await {
        Ok(lista) => {
            info!("GET /ppto/partidas?presupuesto={} ← 200 {} registros", presupuesto, lista.len());
            let items: Vec<Value> = lista.iter().map(partida_json).collect();
            let total = items.len();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::partidas_ppto(&items) {
                    Ok(b)  => render::xlsx_resp(b, &format!("partidas_ppto_{}.xlsx", presupuesto)),
                    Err(e) => render::render_err(e),
                },
                Some("pdf") => {
                    let pdf_items: Vec<Value> = lista.iter().map(|p| json!({
                        "nodo":     p.nodo,
                        "concepto": p.concepto,
                        "unidad":   p.unidad_nombre,
                        "cantidad": p.cantidad.map(|d| d.to_string()),
                        "precio_u": p.precio_u.map(|d| d.to_string()),
                        "importe":  p.importe_calculado.map(|d| d.to_string()),
                        "calculo":  p.calculo,
                        "nivel":    p.nivel,
                    })).collect();
                    match render::pdf::presupuesto(presupuesto, &pdf_items) {
                        Ok(b)  => render::pdf_resp(b, &format!("presupuesto_{}.pdf", presupuesto)),
                        Err(e) => render::render_err(e),
                    }
                }
                _ => (StatusCode::OK, Json(json!({ "partidas": items, "total": total }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -115 => {
            info!("GET /ppto/partidas?presupuesto={} ← 404", presupuesto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /ppto/partidas?presupuesto={} ← 500 codigo={}", presupuesto, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Actualiza Nodo ────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/ppto/partidas/{id}/nodo",
    params(("id" = i32, Path, description = "Id de la partida")),
    request_body = ActualizaNodoInput,
    responses(
        (status = 200, description = "Nodo actualizado",                         body = Value),
        (status = 400, description = "Cancelada (tiene hijos) o error",          body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn actualiza_nodo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<ActualizaNodoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id, ppto = body.ppto, nuevo_nodo = %body.nuevo_nodo, "PUT /ppto/partidas/{}/nodo", id);

    let ret = svc::partidas_actualiza_nodo(&state.postgres, id, body.ppto, &body.nuevo_nodo).await;

    if ret.afectado > 0 {
        info!("PUT /ppto/partidas/{}/nodo ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /ppto/partidas/{}/nodo ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Nuevo Nodo ────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/partidas/nuevo-nodo",
    params(
        ("ppto"  = i32,    Query, description = "Id del presupuesto"),
        ("nodo"  = String, Query, description = "Nodo padre"),
        ("nivel" = i32,    Query, description = "Nivel del nodo"),
    ),
    responses(
        (status = 200, description = "Siguiente nodo calculado",  body = Value),
        (status = 500, description = "Error de base de datos",    body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn nuevo_nodo(
    State(state): State<AppState>,
    Query(q): Query<NuevoNodoQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(ppto = q.ppto, nodo = %q.nodo, nivel = q.nivel, "GET /ppto/partidas/nuevo-nodo");

    match svc::nuevo_nodo_adiciona(&state.postgres, q.ppto, &q.nodo, q.nivel).await {
        Ok(nodo_sig) => {
            info!("GET /ppto/partidas/nuevo-nodo ← 200 nodo={}", nodo_sig);
            (StatusCode::OK, Json(json!({ "nodo": nodo_sig })))
        }
        Err(rc) => {
            error!("GET /ppto/partidas/nuevo-nodo ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Carga 2° Nivel ────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/partidas/nivel2",
    params(
        ("nodo" = i32, Query, description = "Id del nodo raíz"),
        ("ppto" = i32, Query, description = "Id del presupuesto"),
    ),
    responses(
        (status = 200, description = "Partidas del 2° nivel",    body = Value),
        (status = 404, description = "Sin partidas",             body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn carga_2_nivel(
    State(state): State<AppState>,
    Query(q): Query<Carga2NivelQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(nodo = q.nodo, ppto = q.ppto, "GET /ppto/partidas/nivel2");

    match svc::carga_2_nivel(&state.postgres, q.nodo, q.ppto).await {
        Ok(lista) => {
            info!("GET /ppto/partidas/nivel2 ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(partida_json).collect();
            (StatusCode::OK, Json(json!({ "partidas": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -96 => {
            info!("GET /ppto/partidas/nivel2 ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/partidas/nivel2 ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
