// Programa...: handler::reportes::proyecto
// Origen.....: oReportes.cs
//
// Rutas:
//   GET /reportes/proyecto/partidas?presupuesto=&format=
//   GET /reportes/proyecto/arbol?proyecto=&format=
//   GET /reportes/proyecto/audita-xref?presupuesto=&format=
//   GET /reportes/proyecto/totales-ppto?presupuesto=
//   GET /reportes/proyecto/ingresos?proyecto=
//   GET /reportes/proyecto/egresos?proyecto=
//   GET /reportes/proyecto/estado-cuenta?id=
//   GET /reportes/proyecto/avance-obra?proyecto=&format=

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::infrastructure::render;
use crate::services::reportes::proyecto as svc;

#[derive(Debug, Deserialize)]
pub struct PresupuestoQuery {
    pub presupuesto: i32,
    pub format:      Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProyectoQuery {
    pub proyecto: i32,
    pub format:   Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IdQuery {
    pub id: i32,
}

// ── Partidas del presupuesto ──────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/partidas",
    params(
        ("presupuesto" = i32,           Query, description = "Id del presupuesto"),
        ("format"      = Option<String>, Query, description = "xlsx | pdf"),
    ),
    responses(
        (status = 200, description = "Partidas del presupuesto", body = Value),
        (status = 404, description = "Sin partidas",             body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "Reportes"
)]
pub async fn carga_partidas(
    State(state): State<AppState>,
    Query(q): Query<PresupuestoQuery>,
) -> Response {
    debug!(presupuesto = q.presupuesto, "GET /reportes/proyecto/partidas");

    match svc::carga_partidas(&state.postgres, q.presupuesto).await {
        Ok(lista) => {
            info!("GET /reportes/proyecto/partidas?presupuesto={} ← 200 {} registros", q.presupuesto, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "nodo":     r.nodo,
                "concepto": r.concepto,
                "unidad":   r.unidad,
                "cantidad": r.cantidad.map(|d| d.to_string()),
                "precio_u": r.precio_u.map(|d| d.to_string()),
                "importe":  r.importe.map(|d| d.to_string()),
                "calculo":  r.calculo,
                "nivel":    r.nivel,
            })).collect();
            let total = items.len();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::partidas_presupuesto(&items) {
                    Ok(b)  => render::xlsx_resp(b, "partidas_presupuesto.xlsx"),
                    Err(e) => render::render_err(e),
                },
                Some("pdf") => match render::pdf::presupuesto(q.presupuesto, &items) {
                    Ok(b)  => render::pdf_resp(b, &format!("presupuesto_{}.pdf", q.presupuesto)),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "partidas": items, "total": total }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/partidas ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Árbol de tareas del proyecto ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/arbol",
    params(
        ("proyecto" = i32,           Query, description = "Id del proyecto"),
        ("format"   = Option<String>, Query, description = "xlsx"),
    ),
    responses(
        (status = 200, description = "Árbol de tareas del proyecto", body = Value),
        (status = 404, description = "Sin tareas",                   body = Value),
        (status = 500, description = "Error de base de datos",       body = Value),
    ),
    tag = "Reportes"
)]
pub async fn arbol_tareas_proyecto(
    State(state): State<AppState>,
    Query(q): Query<ProyectoQuery>,
) -> Response {
    debug!(proyecto = q.proyecto, "GET /reportes/proyecto/arbol");

    match svc::arbol_tareas_proyecto(&state.postgres, q.proyecto).await {
        Ok(lista) => {
            info!("GET /reportes/proyecto/arbol?proyecto={} ← 200 {} registros", q.proyecto, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "nodo":        r.nodo,
                "nivel":       r.nivel,
                "descripcion": r.descripcion,
                "estado":      r.estado,
                "proyecto":    r.proyecto,
                "importe":     r.importe.map(|d| d.to_string()),
            })).collect();
            let total = items.len();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::arbol_proyecto(&items) {
                    Ok(b)  => render::xlsx_resp(b, "arbol_proyecto.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "arbol": items, "total": total }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/arbol ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Audita XREF ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/audita-xref",
    params(
        ("presupuesto" = i32,           Query, description = "Id del presupuesto a auditar"),
        ("format"      = Option<String>, Query, description = "xlsx"),
    ),
    responses(
        (status = 200, description = "Resultado de auditoría XREF", body = Value),
        (status = 404, description = "Sin registros",               body = Value),
        (status = 500, description = "Error de base de datos",      body = Value),
    ),
    tag = "Reportes"
)]
pub async fn audita_xref(
    State(state): State<AppState>,
    Query(q): Query<PresupuestoQuery>,
) -> Response {
    debug!(presupuesto = q.presupuesto, "GET /reportes/proyecto/audita-xref");

    match svc::audita_xref(&state.postgres, q.presupuesto).await {
        Ok(lista) => {
            info!("GET /reportes/proyecto/audita-xref?presupuesto={} ← 200 {} registros", q.presupuesto, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "nodo":        r.nodo,
                "nivel":       r.nivel,
                "descripcion": r.descripcion,
                "estado":      r.estado,
                "proyecto":    r.proyecto,
                "importe":     r.importe.map(|d| d.to_string()),
            })).collect();
            let total = items.len();
            match q.format.as_deref() {
                Some("xlsx") => match render::xlsx::audita_xref(&items) {
                    Ok(b)  => render::xlsx_resp(b, "audita_xref.xlsx"),
                    Err(e) => render::render_err(e),
                },
                _ => (StatusCode::OK, Json(json!({ "xref": items, "total": total }))).into_response(),
            }
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/audita-xref ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response()
        }
    }
}

// ── Totales del presupuesto ───────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/totales-ppto",
    params(
        ("presupuesto" = i32, Query, description = "Id del presupuesto"),
    ),
    responses(
        (status = 200, description = "Total del presupuesto",  body = Value),
        (status = 404, description = "Presupuesto sin total",  body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Reportes"
)]
pub async fn totales_ppto(
    State(state): State<AppState>,
    Query(q): Query<PresupuestoQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(presupuesto = q.presupuesto, "GET /reportes/proyecto/totales-ppto");

    match svc::totales_ppto(&state.postgres, q.presupuesto).await {
        Ok(total) => {
            info!("GET /reportes/proyecto/totales-ppto?presupuesto={} ← 200 total={}", q.presupuesto, total);
            (StatusCode::OK, Json(json!({ "total": total.to_string() })))
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/totales-ppto ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Ingresos del proyecto ─────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/ingresos",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Ingresos del proyecto",  body = Value),
        (status = 404, description = "Sin ingresos",           body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Reportes"
)]
pub async fn ingresos(
    State(state): State<AppState>,
    Query(q): Query<ProyectoQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = q.proyecto, "GET /reportes/proyecto/ingresos");

    match svc::ingresos(&state.postgres, q.proyecto).await {
        Ok(lista) => {
            info!("GET /reportes/proyecto/ingresos?proyecto={} ← 200 {} registros", q.proyecto, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "fecha":      r.fecha.map(|d| d.to_string()),
                "concepto":   r.concepto,
                "referencia": r.referencia,
                "proyecto":   r.proyecto,
                "monto":      r.monto.map(|m| m.to_string()),
                "usuario":    r.usuario,
            })).collect();
            let total = items.len();
            (StatusCode::OK, Json(json!({ "ingresos": items, "total": total })))
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/ingresos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Egresos del proyecto ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/egresos",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
    ),
    responses(
        (status = 200, description = "Egresos del proyecto",   body = Value),
        (status = 404, description = "Sin egresos",            body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Reportes"
)]
pub async fn egresos(
    State(state): State<AppState>,
    Query(q): Query<ProyectoQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = q.proyecto, "GET /reportes/proyecto/egresos");

    match svc::egresos(&state.postgres, q.proyecto).await {
        Ok(lista) => {
            info!("GET /reportes/proyecto/egresos?proyecto={} ← 200 {} registros", q.proyecto, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "fecha":      r.fecha.map(|d| d.to_string()),
                "concepto":   r.concepto,
                "referencia": r.referencia,
                "proyecto":   r.proyecto,
                "monto":      r.monto.map(|m| m.to_string()),
                "usuario":    r.usuario,
            })).collect();
            let total = items.len();
            (StatusCode::OK, Json(json!({ "egresos": items, "total": total })))
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/egresos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Estado de cuenta del cliente ──────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/estado-cuenta",
    params(
        ("id" = i32, Query, description = "Id del cliente"),
    ),
    responses(
        (status = 200, description = "Estado de cuenta del cliente", body = Value),
        (status = 404, description = "Sin movimientos",              body = Value),
        (status = 500, description = "Error de base de datos",       body = Value),
    ),
    tag = "Reportes"
)]
pub async fn estado_de_cuenta(
    State(state): State<AppState>,
    Query(q): Query<IdQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(id = q.id, "GET /reportes/proyecto/estado-cuenta");

    match svc::estado_de_cuenta(&state.postgres, q.id).await {
        Ok(lista) => {
            info!("GET /reportes/proyecto/estado-cuenta?id={} ← 200 {} registros", q.id, lista.len());
            let items: Vec<Value> = lista.iter().map(|r| json!({
                "fecha":      r.fecha.map(|d| d.to_string()),
                "concepto":   r.concepto,
                "referencia": r.referencia,
                "cargo":      r.cargo.map(|d| d.to_string()),
                "abono":      r.abono.map(|d| d.to_string()),
                "saldo":      r.saldo.map(|d| d.to_string()),
            })).collect();
            let total = items.len();
            (StatusCode::OK, Json(json!({ "movimientos": items, "total": total })))
        }
        Err(rc) if rc.codigo > -75 => {
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /reportes/proyecto/estado-cuenta ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Avance de Obra (PDF) ──────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/reportes/proyecto/avance-obra",
    params(
        ("proyecto" = i32,           Query, description = "Id del proyecto"),
        ("format"   = Option<String>, Query, description = "pdf (default) | json"),
    ),
    responses(
        (status = 200, description = "Avance de obra del proyecto", body = Value),
        (status = 404, description = "Sin movimientos",             body = Value),
        (status = 500, description = "Error de base de datos",      body = Value),
    ),
    tag = "Reportes"
)]
pub async fn avance_obra(
    State(state): State<AppState>,
    Query(q): Query<ProyectoQuery>,
) -> Response {
    debug!(proyecto = q.proyecto, "GET /reportes/proyecto/avance-obra");

    let ing_result = svc::ingresos(&state.postgres, q.proyecto).await;
    let egr_result = svc::egresos(&state.postgres, q.proyecto).await;

    let (ing_lista, egr_lista) = match (ing_result, egr_result) {
        (Ok(i), Ok(e)) => (i, e),
        (Err(rc), _) | (_, Err(rc)) => {
            error!("GET /reportes/proyecto/avance-obra ← 500 codigo={}", rc.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje }))).into_response();
        }
    };

    info!(
        "GET /reportes/proyecto/avance-obra?proyecto={} ← 200 ing={} egr={}",
        q.proyecto, ing_lista.len(), egr_lista.len()
    );

    let ing_items: Vec<Value> = ing_lista.iter().map(|r| json!({
        "fecha":      r.fecha.map(|d| d.to_string()),
        "concepto":   r.concepto,
        "referencia": r.referencia,
        "monto":      r.monto.map(|m| m.to_string()),
    })).collect();

    let egr_items: Vec<Value> = egr_lista.iter().map(|r| json!({
        "fecha":      r.fecha.map(|d| d.to_string()),
        "concepto":   r.concepto,
        "referencia": r.referencia,
        "monto":      r.monto.map(|m| m.to_string()),
    })).collect();

    match q.format.as_deref() {
        Some("json") => {
            (StatusCode::OK, Json(json!({
                "ingresos": ing_items,
                "egresos":  egr_items,
            }))).into_response()
        }
        _ => match render::pdf::avance_obra(q.proyecto, &ing_items, &egr_items) {
            Ok(b)  => render::pdf_resp(b, &format!("avance_obra_{}.pdf", q.proyecto)),
            Err(e) => render::render_err(e),
        },
    }
}
