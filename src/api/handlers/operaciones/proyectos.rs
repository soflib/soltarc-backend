// Programa...: handler::operaciones::proyectos
// Descripción: Endpoints HTTP para proyectos
// Origen.....: oProyectos.cs
//
// Rutas:
//   POST   /operaciones/proyectos                        → alta
//   DELETE /operaciones/proyectos/{id}                   → baja
//   PUT    /operaciones/proyectos                        → cambio
//   GET    /operaciones/proyectos/{id}                   → consulta
//   GET    /operaciones/proyectos?activos=bool           → lista
//   PUT    /operaciones/proyectos/{id}/grupo-usuario     → gpo_usr_proy
//   GET    /operaciones/proyectos/{id}/cliente           → cliente_proy
//   GET    /operaciones/proyectos/{id}/directorio        → dir_proy
//   GET    /operaciones/proyectos/{id}/total-ppto        → total_ppto

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
use tracing::{debug, error, info};

use crate::domain::models::proyectos::Proyectos;
use crate::infrastructure::db::app_state::AppState;
use crate::services::operaciones::proyectos as svc;

// ─────────────────────────────────────────────
// INPUT STRUCTS
// ─────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProyectosInput {
    pub id:           Option<i32>,
    pub tipo:         i32,
    pub nombre:       String,
    pub descripcion:  String,
    pub direccion:    String,
    pub comentarios:  String,
    pub estado:       i32,
    /// Presupuesto en pesos
    #[schema(value_type = f64)]
    pub presupuesto:  f64,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_ini:    String,
    /// Formato: "YYYY-MM-DD HH:MM:SS"
    pub fecha_fin:    String,
    pub asignado:     String,
    pub cliente:      i32,
    pub activo:       bool,
    pub gn_id:        i32,
    pub gn_usr_id:    i32,
    pub dir_imagenes: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GpoUsrInput {
    pub grupo:   i32,
    pub usuario: i32,
}

#[derive(Debug, Deserialize)]
pub struct FiltroActivos {
    pub activos: Option<bool>,
}

// ─────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────

fn parse_input(body: ProyectosInput) -> Result<Proyectos, String> {
    let fecha_ini = NaiveDateTime::parse_from_str(&body.fecha_ini, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("fecha_ini inválida: {e}"))?;
    let fecha_fin = NaiveDateTime::parse_from_str(&body.fecha_fin, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("fecha_fin inválida: {e}"))?;
    let presupuesto = Decimal::try_from(body.presupuesto)
        .map_err(|e| format!("presupuesto inválido: {e}"))?;

    Ok(Proyectos {
        id:           body.id.unwrap_or(0),
        tipo:         body.tipo,
        nombre:       body.nombre,
        descripcion:  body.descripcion,
        direccion:    body.direccion,
        comentarios:  body.comentarios,
        estado:       body.estado,
        presupuesto,
        fecha_ini,
        fecha_fin,
        asignado:     body.asignado,
        cliente:      body.cliente,
        activo:       body.activo,
        gn_id:        body.gn_id,
        gn_usr_id:    body.gn_usr_id,
        dir_imagenes: body.dir_imagenes,
    })
}

fn proyecto_json(p: &Proyectos) -> Value {
    json!({
        "id":           p.id,
        "tipo":         p.tipo,
        "nombre":       p.nombre,
        "descripcion":  p.descripcion,
        "direccion":    p.direccion,
        "comentarios":  p.comentarios,
        "estado":       p.estado,
        "presupuesto":  p.presupuesto.to_string(),
        "fecha_ini":    p.fecha_ini.format("%Y-%m-%d %H:%M:%S").to_string(),
        "fecha_fin":    p.fecha_fin.format("%Y-%m-%d %H:%M:%S").to_string(),
        "asignado":     p.asignado,
        "cliente":      p.cliente,
        "activo":       p.activo,
        "gn_id":        p.gn_id,
        "gn_usr_id":    p.gn_usr_id,
        "dir_imagenes": p.dir_imagenes,
    })
}

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/proyectos",
    request_body = ProyectosInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<ProyectosInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /operaciones/proyectos → nombre='{}'", body.nombre);

    let proy = match parse_input(body) {
        Ok(p) => p,
        Err(msg) => {
            error!("POST /operaciones/proyectos ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &proy).await;

    if ret.afectado > 0 {
        info!("POST /operaciones/proyectos ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED,     Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /operaciones/proyectos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/operaciones/proyectos/{id}",
    params(("id" = i32, Path, description = "Id del proyecto a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /operaciones/proyectos/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /operaciones/proyectos/{} ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /operaciones/proyectos/{} ← 400 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIO
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/operaciones/proyectos",
    request_body = ProyectosInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Json(body): Json<ProyectosInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /operaciones/proyectos → id={:?} nombre='{}'", body.id, body.nombre);

    let Some(_) = body.id else {
        error!("PUT /operaciones/proyectos ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })),
        );
    };

    let proy = match parse_input(body) {
        Ok(p) => p,
        Err(msg) => {
            error!("PUT /operaciones/proyectos ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambio(&state.postgres, &proy).await;

    if ret.afectado > 0 {
        info!("PUT /operaciones/proyectos ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /operaciones/proyectos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/{id}",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/proyectos/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(p)) => {
            info!("GET /operaciones/proyectos/{} ← 200 nombre='{}'", id, p.nombre);
            (StatusCode::OK, Json(proyecto_json(&p)))
        }
        Ok(None) => {
            info!("GET /operaciones/proyectos/{} ← 404", id);
            (StatusCode::NOT_FOUND,            Json(json!({ "codigo": -41, "mensaje": "No existe el proyecto" })))
        }
        Err(ret) => {
            error!("GET /operaciones/proyectos/{} ← 500 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// LISTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos",
    params(("activos" = Option<bool>, Query, description = "true = sólo activos (default), false = todos")),
    responses(
        (status = 200, description = "Lista de proyectos",      body = Value),
        (status = 404, description = "Sin registros",           body = Value),
        (status = 500, description = "Error de base de datos",  body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn lista(
    State(state): State<AppState>,
    Query(filtro): Query<FiltroActivos>,
) -> (StatusCode, Json<Value>) {
    let activos = filtro.activos.unwrap_or(true);
    debug!("GET /operaciones/proyectos?activos={}", activos);

    match svc::llena_proyectos(&state.postgres, activos).await {
        Ok(lista) => {
            info!("GET /operaciones/proyectos ← 200 {} proyectos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|p| json!({
                "id":          p.id,
                "nombre":      p.nombre,
                "estado":      p.estado,
                "presupuesto": p.presupuesto.to_string(),
                "fecha_ini":   p.fecha_ini.format("%Y-%m-%d").to_string(),
                "fecha_fin":   p.fecha_fin.format("%Y-%m-%d").to_string(),
                "cliente":     p.cliente,
                "activo":      p.activo,
                "gn_id":       p.gn_id,
                "gn_usr_id":   p.gn_usr_id,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/proyectos ← 500 codigo={} msg='{}'", ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/proyectos ← 404 sin registros");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// GPO USR PROYECTO
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/operaciones/proyectos/{id}/grupo-usuario",
    params(("id" = i32, Path, description = "Id del proyecto")),
    request_body = GpoUsrInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn gpo_usr_proy(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(body): Json<GpoUsrInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /operaciones/proyectos/{}/grupo-usuario → grupo={} usuario={}", id, body.grupo, body.usuario);

    let ret = svc::gpo_usr_proyecto(&state.postgres, id, body.grupo, body.usuario).await;

    if ret.afectado > 0 {
        info!("PUT /operaciones/proyectos/{}/grupo-usuario ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /operaciones/proyectos/{}/grupo-usuario ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CLIENTE PROYECTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/{id}/cliente",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Nombre del cliente", body = Value),
        (status = 404, description = "Sin cliente",        body = Value),
        (status = 500, description = "Error BD",           body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn cliente_proy(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/proyectos/{}/cliente", id);

    match svc::cliente_proyecto(&state.postgres, id).await {
        Ok(nombre) => {
            info!("GET /operaciones/proyectos/{}/cliente ← 200 '{}'", id, nombre);
            (StatusCode::OK, Json(json!({ "proyecto_id": id, "cliente": nombre })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/proyectos/{}/cliente ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/proyectos/{}/cliente ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// DIR PROYECTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/{id}/directorio",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Directorio de imágenes", body = Value),
        (status = 404, description = "Sin directorio",         body = Value),
        (status = 500, description = "Error BD",               body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn dir_proy(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/proyectos/{}/directorio", id);

    match svc::dir_proyecto(&state.postgres, id).await {
        Ok(dir) => {
            info!("GET /operaciones/proyectos/{}/directorio ← 200", id);
            (StatusCode::OK, Json(json!({ "proyecto_id": id, "directorio": dir })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/proyectos/{}/directorio ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/proyectos/{}/directorio ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// LISTA GRUPOS DE NEGOCIO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/grupos",
    responses(
        (status = 200, description = "Lista de grupos activos", body = Value),
        (status = 404, description = "Sin registros",           body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn lista_grupos(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match svc::lista_grupos(&state.postgres).await {
        Ok(lista) => (StatusCode::OK, Json(json!(
            lista.iter().map(|g| json!({ "id": g.id, "nombre": g.nombre })).collect::<Vec<_>>()
        ))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!([]))),
    }
}

// ─────────────────────────────────────────────
// USUARIOS DE UN GRUPO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/grupos/{id}/usuarios",
    params(("id" = i32, Path, description = "Id del grupo de negocio")),
    responses(
        (status = 200, description = "Lista de usuarios del grupo", body = Value),
        (status = 404, description = "Sin registros",               body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn usuarios_grupo(
    State(state): State<AppState>,
    Path(grupo_id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    match svc::usuarios_grupo(&state.postgres, grupo_id).await {
        Ok(lista) => (StatusCode::OK, Json(json!(
            lista.iter().map(|u| json!({ "id": u.id, "user_id": u.user_id })).collect::<Vec<_>>()
        ))),
        Err(_) => (StatusCode::NOT_FOUND, Json(json!([]))),
    }
}

// ─────────────────────────────────────────────
// TOTAL PPTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/{id}/total-ppto",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Total presupuesto", body = Value),
        (status = 404, description = "Sin datos",         body = Value),
        (status = 500, description = "Error BD",          body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn total_ppto(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/proyectos/{}/total-ppto", id);

    match svc::total_ppto(&state.postgres, id).await {
        Ok(total) => {
            info!("GET /operaciones/proyectos/{}/total-ppto ← 200 total={}", id, total);
            (StatusCode::OK, Json(json!({ "proyecto_id": id, "total_ppto": total.to_string() })))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /operaciones/proyectos/{}/total-ppto ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /operaciones/proyectos/{}/total-ppto ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
