// Programa...: handler::operaciones::xref
// Descripción: Endpoints HTTP para XRef detalle proyecto ↔ finanzas
// Origen.....: oXref_DetalleProy_Finan.cs
//
// Rutas:
//   POST   /operaciones/xref                         → alta
//   PUT    /operaciones/xref                         → cambio
//   DELETE /operaciones/xref/{id}                    → baja
//   GET    /operaciones/xref/{id}                    → consulta
//   GET    /operaciones/xref/{id}/egresos            → egresos_a_partidas (by partida)
//   GET    /operaciones/xref/no-asignados?proyecto=  → egresos_no_asignados

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::xref_detalle_proy_finan::XrefDetalleProyFinan;
use crate::infrastructure::db::app_state::AppState;
use crate::services::operaciones::xref as svc;

// ─────────────────────────────────────────────
// INPUT STRUCTS
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct XrefInput {
    pub id:           Option<i32>,
    pub partida:      i32,
    pub tipo:         i32,
    pub transaccion:  i32,
    pub comentario:   String,
    pub proyecto:     i32,
    #[schema(value_type = f64)]
    pub monto_aplica: f64,
}

#[derive(Debug, Deserialize)]
pub struct FiltroProyecto {
    pub proyecto: Option<i32>,
}

// ─────────────────────────────────────────────
// HELPER
// ─────────────────────────────────────────────

fn parse_input(body: XrefInput) -> Result<XrefDetalleProyFinan, String> {
    let monto_aplica = Decimal::try_from(body.monto_aplica)
        .map_err(|e| format!("monto_aplica inválido: {e}"))?;
    Ok(XrefDetalleProyFinan {
        id:           body.id.unwrap_or(0),
        partida:      body.partida,
        tipo:         body.tipo,
        transaccion:  body.transaccion,
        comentario:   body.comentario,
        proyecto:     body.proyecto,
        monto_aplica,
    })
}

fn xref_json(x: &XrefDetalleProyFinan) -> Value {
    json!({
        "id":           x.id,
        "partida":      x.partida,
        "tipo":         x.tipo,
        "transaccion":  x.transaccion,
        "comentario":   x.comentario,
        "proyecto":     x.proyecto,
        "monto_aplica": x.monto_aplica.to_string(),
    })
}

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/xref",
    request_body = XrefInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "XRef"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<XrefInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /operaciones/xref → partida={} egreso={}", body.partida, body.transaccion);

    let xref = match parse_input(body) {
        Ok(x) => x,
        Err(msg) => {
            error!("POST /operaciones/xref ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &xref).await;

    if ret.afectado > 0 {
        info!("POST /operaciones/xref ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /operaciones/xref ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/operaciones/xref/{id}",
    params(("id" = i32, Path, description = "Id del xref a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "XRef"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /operaciones/xref/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /operaciones/xref/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /operaciones/xref/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIO
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/operaciones/xref",
    request_body = XrefInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "XRef"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Json(body): Json<XrefInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /operaciones/xref → id={:?}", body.id);

    let Some(_) = body.id else {
        error!("PUT /operaciones/xref ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })),
        );
    };

    let xref = match parse_input(body) {
        Ok(x) => x,
        Err(msg) => {
            error!("PUT /operaciones/xref ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambio(&state.postgres, &xref).await;

    if ret.afectado > 0 {
        info!("PUT /operaciones/xref ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /operaciones/xref ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/xref/{id}",
    params(("id" = i32, Path, description = "Id del xref")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "XRef"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/xref/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(x)) => {
            info!("GET /operaciones/xref/{} ← 200", id);
            (StatusCode::OK, Json(xref_json(&x)))
        }
        Ok(None) => {
            info!("GET /operaciones/xref/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "No existe el registro xref" })))
        }
        Err(ret) => {
            error!("GET /operaciones/xref/{} ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// EGRESOS A PARTIDAS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/xref/{id}/egresos",
    params(("id" = i32, Path, description = "Id de la partida")),
    responses(
        (status = 200, description = "Lista de egresos asignados a la partida", body = Value),
        (status = 404, description = "Sin egresos para esta partida",           body = Value),
        (status = 500, description = "Error de base de datos",                  body = Value),
    ),
    tag = "XRef"
)]
pub async fn egresos_a_partidas(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/xref/{}/egresos", id);

    match svc::egresos_a_partidas(&state.postgres, id).await {
        Ok(lista) => {
            info!("GET /operaciones/xref/{}/egresos ← 200 {} registros", id, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|x| xref_json(x)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/xref/{}/egresos ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/xref/{}/egresos ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// EGRESOS NO ASIGNADOS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/xref/no-asignados",
    params(("proyecto" = i32, Query, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Egresos no asignados a ninguna partida", body = Value),
        (status = 404, description = "No hay egresos sin asignar",             body = Value),
        (status = 500, description = "Error de base de datos",                 body = Value),
    ),
    tag = "XRef"
)]
pub async fn egresos_no_asignados(
    State(state): State<AppState>,
    Query(filtro): Query<FiltroProyecto>,
) -> (StatusCode, Json<Value>) {
    let proyecto = match filtro.proyecto {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El parámetro proyecto es requerido" })));
        }
    };
    debug!("GET /operaciones/xref/no-asignados?proyecto={}", proyecto);

    match svc::egresos_no_asignados(&state.postgres, proyecto).await {
        Ok(lista) => {
            info!("GET /operaciones/xref/no-asignados?proyecto={} ← 200 {} registros", proyecto, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|x| xref_json(x)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/xref/no-asignados ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/xref/no-asignados?proyecto={} ← 404", proyecto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
