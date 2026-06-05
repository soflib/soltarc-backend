// Programa...: handler::operaciones::detalle_proyecto
// Descripción: Endpoints HTTP para detalle de proyectos
// Origen.....: oDetalleProyecto.cs
//
// Rutas:
//   POST   /operaciones/detalle-proyecto                    → alta
//   PUT    /operaciones/detalle-proyecto                    → cambios
//   DELETE /operaciones/detalle-proyecto/{id}               → baja
//   GET    /operaciones/detalle-proyecto/{id}               → consulta
//   GET    /operaciones/detalle-proyecto?proyecto=          → partidas_proyecto
//   GET    /operaciones/detalle-proyecto/{proyecto}/tareas  → carga_tareas
//   GET    /operaciones/detalle-proyecto/nodos-desc?nodo=   → nodos_desc
//   PUT    /operaciones/detalle-proyecto/fechas             → actualiza_fechas
//   POST   /operaciones/detalle-proyecto/copia              → copia_contenido_partidas
//   POST   /operaciones/detalle-proyecto/adiciona           → adiciona_partidas_faltantes

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::detalle_proyectos::{DetalleProyectos, NodoArbol};
use crate::infrastructure::db::app_state::AppState;
use crate::services::operaciones::detalle_proyecto as svc;

#[derive(Debug, Deserialize)]
pub struct ArbolQuery {
    pub proyecto: i32,
}

#[derive(Debug, Deserialize)]
pub struct BuscarQuery {
    pub proyecto: i32,
    pub q:        Option<String>,
}

// ─────────────────────────────────────────────
// INPUT STRUCTS
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct DetalleProyectosInput {
    pub id:          Option<i32>,
    pub proyecto:    i32,
    pub tipo:        i32,
    pub secuencia:   i32,
    pub descripcion: String,
    pub comentarios: String,
    #[schema(value_type = f64)]
    pub presupuesto: f64,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_inicio: String,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_fin:    String,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_termina: String,
    pub estado:      i32,
    pub nodo:        String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ActualizaFechasInput {
    pub proyecto:      i32,
    pub nodo:          String,
    /// Formato: "YYYY-MM-DD"
    pub fecha_ini:     String,
    /// Formato: "YYYY-MM-DD"
    pub fecha_fin:     String,
    pub estado:        i32,
    /// Formato: "YYYY-MM-DD"
    pub fecha_termino: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CopiaPartidaInput {
    pub origen:  i32,
    pub destino: i32,
}

#[derive(Debug, Deserialize)]
pub struct FiltroProyecto {
    pub proyecto: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FiltroNodo {
    pub proyecto: Option<i32>,
    pub nodo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FiltroCopiaQry {
    pub pry_origen:  Option<i32>,
    pub pry_destino: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FiltroNivel2 {
    pub proyecto: Option<i32>,
    pub nivel:    Option<i32>,
}

// ─────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────

fn parse_datetime(s: &str, field: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("{} inválido: {e}", field))
}

fn parse_input(body: DetalleProyectosInput) -> Result<DetalleProyectos, String> {
    let fecha_inicio  = parse_datetime(&body.fecha_inicio,  "fecha_inicio")?;
    let fecha_fin     = parse_datetime(&body.fecha_fin,     "fecha_fin")?;
    let fecha_termina = parse_datetime(&body.fecha_termina, "fecha_termina")?;
    let presupuesto   = Decimal::try_from(body.presupuesto)
        .map_err(|e| format!("presupuesto inválido: {e}"))?;

    Ok(DetalleProyectos {
        id:           body.id.unwrap_or(0),
        proyecto:     body.proyecto,
        tipo:         body.tipo,
        secuencia:    body.secuencia,
        descripcion:  body.descripcion,
        comentarios:  body.comentarios,
        presupuesto,
        fecha_inicio,
        fecha_fin,
        fecha_termina,
        estado:       body.estado,
        nodo:         body.nodo,
    })
}

fn det_json(d: &DetalleProyectos) -> Value {
    json!({
        "id":           d.id,
        "proyecto":     d.proyecto,
        "tipo":         d.tipo,
        "secuencia":    d.secuencia,
        "descripcion":  d.descripcion,
        "comentarios":  d.comentarios,
        "presupuesto":  d.presupuesto.to_string(),
        "fecha_inicio":  d.fecha_inicio.format("%Y-%m-%d %H:%M:%S").to_string(),
        "fecha_fin":     d.fecha_fin.format("%Y-%m-%d %H:%M:%S").to_string(),
        "fecha_termina": d.fecha_termina.format("%Y-%m-%d %H:%M:%S").to_string(),
        "estado":       d.estado,
        "nodo":         d.nodo,
    })
}

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/detalle-proyecto",
    request_body = DetalleProyectosInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<DetalleProyectosInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /operaciones/detalle-proyecto → proyecto={} desc='{}'", body.proyecto, body.descripcion);

    let det = match parse_input(body) {
        Ok(d) => d,
        Err(msg) => {
            error!("POST /operaciones/detalle-proyecto ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &det).await;

    if ret.afectado > 0 {
        info!("POST /operaciones/detalle-proyecto ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /operaciones/detalle-proyecto ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/operaciones/detalle-proyecto/{id}",
    params(("id" = i32, Path, description = "Id del detalle a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /operaciones/detalle-proyecto/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /operaciones/detalle-proyecto/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /operaciones/detalle-proyecto/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/operaciones/detalle-proyecto",
    request_body = DetalleProyectosInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<DetalleProyectosInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /operaciones/detalle-proyecto → id={:?}", body.id);

    let Some(_) = body.id else {
        error!("PUT /operaciones/detalle-proyecto ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })),
        );
    };

    let det = match parse_input(body) {
        Ok(d) => d,
        Err(msg) => {
            error!("PUT /operaciones/detalle-proyecto ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambios(&state.postgres, &det).await;

    if ret.afectado > 0 {
        info!("PUT /operaciones/detalle-proyecto ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /operaciones/detalle-proyecto ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/{id}",
    params(("id" = i32, Path, description = "Id del detalle")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/detalle-proyecto/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(d)) => {
            info!("GET /operaciones/detalle-proyecto/{} ← 200", id);
            (StatusCode::OK, Json(det_json(&d)))
        }
        Ok(None) => {
            info!("GET /operaciones/detalle-proyecto/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "No existe el detalle" })))
        }
        Err(ret) => {
            error!("GET /operaciones/detalle-proyecto/{} ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// PARTIDAS PROYECTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto",
    params(("proyecto" = i32, Query, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Lista de partidas",       body = Value),
        (status = 404, description = "Sin partidas",            body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn partidas_proyecto(
    State(state): State<AppState>,
    Query(filtro): Query<FiltroProyecto>,
) -> (StatusCode, Json<Value>) {
    let proyecto = match filtro.proyecto {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El parámetro proyecto es requerido" })));
        }
    };
    debug!("GET /operaciones/detalle-proyecto?proyecto={}", proyecto);

    match svc::partidas_proyecto(&state.postgres, proyecto).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto?proyecto={} ← 200 {} registros", proyecto, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|d| det_json(d)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/detalle-proyecto?proyecto={} ← 500 codigo={}", proyecto, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/detalle-proyecto?proyecto={} ← 404", proyecto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CARGA TAREAS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/{proyecto}/tareas",
    params(("proyecto" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Lista de tareas",         body = Value),
        (status = 404, description = "Sin tareas",              body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn carga_tareas(
    State(state): State<AppState>,
    Path(proyecto): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/detalle-proyecto/{}/tareas", proyecto);

    match svc::carga_tareas(&state.postgres, proyecto).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/{}/tareas ← 200 {} tareas", proyecto, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|d| det_json(d)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/detalle-proyecto/{}/tareas ← 500 codigo={}", proyecto, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/detalle-proyecto/{}/tareas ← 404", proyecto);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// NODOS DESC
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/nodos-desc",
    params(("nodo" = String, Query, description = "Nodo raíz (ej: /1/2/)")),
    responses(
        (status = 200, description = "Nodos descendientes",     body = Value),
        (status = 404, description = "Sin nodos",               body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn nodos_desc(
    State(state): State<AppState>,
    Query(filtro): Query<FiltroNodo>,
) -> (StatusCode, Json<Value>) {
    let nodo = match filtro.nodo {
        Some(n) => n,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El parámetro nodo es requerido" })));
        }
    };
    let proyecto = match filtro.proyecto {
        Some(p) => p,
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El parámetro proyecto es requerido" })));
        }
    };
    debug!("GET /operaciones/detalle-proyecto/nodos-desc?proyecto={}&nodo={}", proyecto, nodo);

    match svc::nodos_desc(&state.postgres, proyecto, &nodo).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/nodos-desc ← 200 {} nodos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|d| det_json(d)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/detalle-proyecto/nodos-desc ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/detalle-proyecto/nodos-desc ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// ACTUALIZA FECHAS
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/operaciones/detalle-proyecto/fechas",
    request_body = ActualizaFechasInput,
    responses(
        (status = 200, description = "Fechas actualizadas",              body = Value),
        (status = 400, description = "Error de validación o cancelado",  body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn actualiza_fechas(
    State(state): State<AppState>,
    Json(body): Json<ActualizaFechasInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /operaciones/detalle-proyecto/fechas → nodo='{}'", body.nodo);

    let fmt = time::macros::format_description!("[year]-[month]-[day]");

    let fecha_ini = time::Date::parse(&body.fecha_ini, fmt)
        .map_err(|e| format!("fecha_ini inválida: {e}"));
    let fecha_fin = time::Date::parse(&body.fecha_fin, fmt)
        .map_err(|e| format!("fecha_fin inválida: {e}"));
    let fecha_termino = time::Date::parse(&body.fecha_termino, fmt)
        .map_err(|e| format!("fecha_termino inválida: {e}"));

    let (fi, ff, ft) = match (fecha_ini, fecha_fin, fecha_termino) {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            error!("PUT /operaciones/detalle-proyecto/fechas ← 400 parse: {}", e);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": e })));
        }
    };

    let ret = svc::actualiza_fechas(&state.postgres, body.proyecto, &body.nodo, fi, ff, body.estado, ft).await;

    if ret.afectado > 0 {
        info!("PUT /operaciones/detalle-proyecto/fechas ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /operaciones/detalle-proyecto/fechas ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// COPIA CONTENIDO PARTIDAS
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/detalle-proyecto/copia",
    request_body = CopiaPartidaInput,
    responses(
        (status = 200, description = "Copia realizada",                  body = Value),
        (status = 400, description = "Sin partidas para copiar o error", body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn copia_contenido_partidas(
    State(state): State<AppState>,
    Json(body): Json<CopiaPartidaInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /operaciones/detalle-proyecto/copia → origen={} destino={}", body.origen, body.destino);

    let ret = svc::copia_contenido_partidas(&state.postgres, body.origen, body.destino).await;

    if ret.afectado > 0 {
        info!("POST /operaciones/detalle-proyecto/copia ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /operaciones/detalle-proyecto/copia ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// ADICIONA PARTIDAS FALTANTES
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/detalle-proyecto/adiciona",
    request_body = CopiaPartidaInput,
    responses(
        (status = 200, description = "Partidas adicionadas",             body = Value),
        (status = 400, description = "Sin partidas para agregar o error", body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn adiciona_partidas_faltantes(
    State(state): State<AppState>,
    Json(body): Json<CopiaPartidaInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /operaciones/detalle-proyecto/adiciona → origen={} destino={}", body.origen, body.destino);

    let ret = svc::adiciona_partidas_faltantes(&state.postgres, body.origen, body.destino).await;

    if ret.afectado > 0 {
        info!("POST /operaciones/detalle-proyecto/adiciona ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /operaciones/detalle-proyecto/adiciona ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── CSV import ────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/operaciones/detalle-proyecto/import-csv",
    params(("proyecto" = i32, Query, description = "Id del proyecto destino")),
    responses(
        (status = 200, description = "CSV importado",              body = Value),
        (status = 400, description = "Error de parseo o validación", body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn import_csv(
    State(state): State<AppState>,
    Query(q): Query<FiltroProyecto>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let proyecto_id = match q.proyecto {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Parámetro 'proyecto' requerido" }))),
    };
    info!("POST /operaciones/detalle-proyecto/import-csv proyecto={}", proyecto_id);

    let mut csv_bytes: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("archivo") {
            match field.bytes().await {
                Ok(b) => csv_bytes = Some(b.to_vec()),
                Err(e) => {
                    error!("import_csv: error leyendo multipart — {}", e);
                    return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": format!("Error al leer el archivo: {}", e) })));
                }
            }
            break;
        }
    }

    let bytes = match csv_bytes {
        Some(b) if !b.is_empty() => b,
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "Campo 'archivo' no encontrado o vacío" }))),
    };

    let (inserted, errors) = svc::import_csv(&state.postgres, proyecto_id, &bytes).await;

    info!(
        "POST /operaciones/detalle-proyecto/import-csv ← insertados={} errores={}",
        inserted, errors.len()
    );

    (StatusCode::OK, Json(json!({
        "insertados": inserted,
        "errores":    errors,
    })))
}

// ─────────────────────────────────────────────
// PART NO DESTINO — sp_cpa_DetProyADDQry
// Partidas en origen que NO existen en destino
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/no-destino",
    params(
        ("pry_origen"  = i32, Query, description = "Id proyecto origen"),
        ("pry_destino" = i32, Query, description = "Id proyecto destino"),
    ),
    responses(
        (status = 200, description = "Nodos faltantes",          body = Value),
        (status = 404, description = "Sin diferencias",          body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn part_no_destino(
    State(state): State<AppState>,
    Query(q): Query<FiltroCopiaQry>,
) -> (StatusCode, Json<Value>) {
    let (origen, destino) = match (q.pry_origen, q.pry_destino) {
        (Some(a), Some(b)) => (a, b),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "pry_origen y pry_destino son requeridos" }))),
    };
    debug!("GET /operaciones/detalle-proyecto/no-destino origen={} destino={}", origen, destino);

    match svc::adiciona_partidas_qry(&state.postgres, origen, destino).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/no-destino ← 200 {} nodos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|d| det_json(d)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/detalle-proyecto/no-destino ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/detalle-proyecto/no-destino ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CARGA 2° NIVEL
// Partidas del proyecto filtradas por profundidad de nodo
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/nivel2",
    params(
        ("proyecto" = i32, Query, description = "Id del proyecto"),
        ("nivel"    = i32, Query, description = "Nivel jerárquico (1 = raíz, 2 = hijos, …)"),
    ),
    responses(
        (status = 200, description = "Partidas del nivel solicitado", body = Value),
        (status = 404, description = "Sin partidas en ese nivel",     body = Value),
        (status = 500, description = "Error de base de datos",        body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn carga_nivel(
    State(state): State<AppState>,
    Query(q): Query<FiltroNivel2>,
) -> (StatusCode, Json<Value>) {
    let (proyecto, nivel) = match (q.proyecto, q.nivel) {
        (Some(p), Some(n)) => (p, n),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "proyecto y nivel son requeridos" }))),
    };
    debug!("GET /operaciones/detalle-proyecto/nivel2 proyecto={} nivel={}", proyecto, nivel);

    match svc::carga_nivel(&state.postgres, proyecto, nivel).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/nivel2 ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|d| det_json(d)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/detalle-proyecto/nivel2 ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/detalle-proyecto/nivel2 ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// COPIA PART QRY — sp_cpa_DetProyCopyQry
// Comparación entre nodos de proyecto origen y destino
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/copia-qry",
    params(
        ("pry_origen"  = i32, Query, description = "Id proyecto origen"),
        ("pry_destino" = i32, Query, description = "Id proyecto destino"),
    ),
    responses(
        (status = 200, description = "Diferencias entre proyectos", body = Value),
        (status = 404, description = "Sin diferencias",             body = Value),
        (status = 500, description = "Error de base de datos",      body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn copia_part_qry(
    State(state): State<AppState>,
    Query(q): Query<FiltroCopiaQry>,
) -> (StatusCode, Json<Value>) {
    let (origen, destino) = match (q.pry_origen, q.pry_destino) {
        (Some(a), Some(b)) => (a, b),
        _ => return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "pry_origen y pry_destino son requeridos" }))),
    };
    debug!("GET /operaciones/detalle-proyecto/copia-qry origen={} destino={}", origen, destino);

    match svc::copia_cont_partidas_qry(&state.postgres, origen, destino).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/copia-qry ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|d| det_json(d)).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/detalle-proyecto/copia-qry ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/detalle-proyecto/copia-qry ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ── Árbol WBS del proyecto ────────────────────────────────────────────────────
//
// GET /operaciones/detalle-proyecto/arbol?proyecto=X
// Devuelve id, nodo, descripción, nivel, importe, estado para que el frontend
// reconstruya la jerarquía completa.

#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/arbol",
    params(("proyecto" = i32, Query, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Árbol del WBS",            body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn arbol(
    State(state): State<AppState>,
    Query(q): Query<ArbolQuery>,
) -> (StatusCode, Json<Value>) {
    debug!(proyecto = q.proyecto, "GET /operaciones/detalle-proyecto/arbol");

    match svc::arbol(&state.postgres, q.proyecto).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/arbol?proyecto={} ← 200 {} nodos", q.proyecto, lista.len());
            let payload: Vec<NodoArbol> = lista;
            (StatusCode::OK, Json(json!({ "items": payload, "total": payload.len() })))
        }
        Err(rc) => {
            error!("GET /operaciones/detalle-proyecto/arbol?proyecto={} ← 500 codigo={}", q.proyecto, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Búsqueda WBS por descripción ──────────────────────────────────────────────
//
// GET /operaciones/detalle-proyecto/buscar?proyecto=X&q=texto
// Devuelve coincidencias con `ruta` ancestral.

#[utoipa::path(
    get,
    path = "/operaciones/detalle-proyecto/buscar",
    params(
        ("proyecto" = i32,            Query, description = "Id del proyecto"),
        ("q"        = Option<String>, Query, description = "Texto a buscar en la descripción (ILIKE)"),
    ),
    responses(
        (status = 200, description = "Lista de coincidencias con ruta", body = Value),
        (status = 500, description = "Error de base de datos",          body = Value),
    ),
    tag = "DetalleProyecto"
)]
pub async fn buscar(
    State(state): State<AppState>,
    Query(q): Query<BuscarQuery>,
) -> (StatusCode, Json<Value>) {
    let texto = q.q.unwrap_or_default();
    debug!(proyecto = q.proyecto, q = %texto, "GET /operaciones/detalle-proyecto/buscar");

    match svc::buscar(&state.postgres, q.proyecto, &texto).await {
        Ok(lista) => {
            info!("GET /operaciones/detalle-proyecto/buscar ← 200 {} items", lista.len());
            let payload: Vec<NodoArbol> = lista;
            (StatusCode::OK, Json(json!({ "items": payload, "total": payload.len() })))
        }
        Err(rc) => {
            error!("GET /operaciones/detalle-proyecto/buscar ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
