// Programa...: handler::catalog_g::clients
// Descripción: Endpoints HTTP para el catálogo de clientes
// Origen.....: Clientes.aspx.cs
//
// Rutas:
//   POST   /catalog/clients           → alta
//   DELETE /catalog/clients/{id}      → baja
//   PUT    /catalog/clients           → cambios
//   GET    /catalog/clients/{id}      → consulta
//   GET    /catalog/clients/{id}/nombre → nombre_cliente  (C# NomCte compat)
//   GET    /catalog/clients           → obtiene_clientes  (?activos=bool, default true)
//   GET    /catalog/clients/tipos     → obtiene_tipos     (catálogo tipo 3)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::clients::Clientes;
use crate::infrastructure::db::app_state::AppState;
use crate::services::catalog_g::clients as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClienteInput {
    pub id:           Option<i32>,
    pub nombre:       String,
    pub direccion:    String,
    pub telefono:     String,
    pub mail:         String,
    pub cuenta_banco: String,
    pub comentarios:  String,
    pub tipo:         i32,
    pub activo:       bool,
    pub condiciones:  String,
}

#[derive(Debug, Deserialize)]
pub struct FiltroActivos {
    pub activos: Option<bool>,
}

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/catalog/clients",
    request_body = ClienteInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "Clients"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<ClienteInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /catalog/clients → nombre='{}' tipo={}", body.nombre, body.tipo);

    let cte = Clientes {
        id:           body.id.unwrap_or(0),
        nombre:       body.nombre,
        direccion:    body.direccion,
        telefono:     body.telefono,
        mail:         body.mail,
        cuenta_banco: body.cuenta_banco,
        comentarios:  body.comentarios,
        tipo:         body.tipo,
        tipo_nombre:  None, // poblado por SP en lecturas
        activo:       body.activo,
        condiciones:  body.condiciones,
    };
    let ret = svc::alta(&state.postgres, &cte).await;

    if ret.afectado > 0 {
        info!("POST /catalog/clients ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED,     Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /catalog/clients ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/catalog/clients/{id}",
    params(("id" = i32, Path, description = "Id del cliente a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "Clients"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /catalog/clients/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /catalog/clients/{} ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /catalog/clients/{} ← 400 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/catalog/clients",
    request_body = ClienteInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Clients"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<ClienteInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /catalog/clients → id={:?} nombre='{}'", body.id, body.nombre);

    let Some(id) = body.id else {
        error!("PUT /catalog/clients ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })),
        );
    };
    let cte = Clientes {
        id,
        nombre:       body.nombre,
        direccion:    body.direccion,
        telefono:     body.telefono,
        mail:         body.mail,
        cuenta_banco: body.cuenta_banco,
        comentarios:  body.comentarios,
        tipo:         body.tipo,
        tipo_nombre:  None,
        activo:       body.activo,
        condiciones:  body.condiciones,
    };
    let ret = svc::cambios(&state.postgres, &cte).await;

    if ret.afectado > 0 {
        info!("PUT /catalog/clients ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /catalog/clients ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/clients/{id}",
    params(("id" = i32, Path, description = "Id del cliente a consultar")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Clients"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/clients/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(c)) => {
            info!("GET /catalog/clients/{} ← 200 nombre='{}'", id, c.nombre);
            (StatusCode::OK, Json(json!({
                "id":           c.id,
                "nombre":       c.nombre,
                "direccion":    c.direccion,
                "telefono":     c.telefono,
                "mail":         c.mail,
                "cuenta_banco": c.cuenta_banco,
                "comentarios":  c.comentarios,
                "tipo":         c.tipo,
                "tipo_nombre":  c.tipo_nombre,
                "activo":       c.activo,
                "condiciones":  c.condiciones,
            })))
        }
        Ok(None) => {
            info!("GET /catalog/clients/{} ← 404", id);
            (StatusCode::NOT_FOUND,            Json(json!({ "codigo": -41, "mensaje": "No existe el registro" })))
        }
        Err(ret) => {
            error!("GET /catalog/clients/{} ← 500 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE CLIENTES
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/clients",
    params(("activos" = Option<bool>, Query, description = "true = sólo activos (default), false = todos")),
    responses(
        (status = 200, description = "Lista de clientes",    body = Value),
        (status = 404, description = "Sin registros",        body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Clients"
)]
pub async fn obtiene_clientes(
    State(state): State<AppState>,
    Query(filtro): Query<FiltroActivos>,
) -> (StatusCode, Json<Value>) {
    let activos = filtro.activos.unwrap_or(true);
    debug!("GET /catalog/clients?activos={}", activos);

    match svc::obtiene_clientes(&state.postgres, activos).await {
        Ok(lista) => {
            info!("GET /catalog/clients ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|c| json!({
                "id":          c.id,
                "nombre":      c.nombre,
                "mail":        c.mail,
                "telefono":    c.telefono,
                "tipo":        c.tipo,
                "tipo_nombre": c.tipo_nombre,
                "activo":      c.activo,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /catalog/clients ← 500 codigo={} msg='{}'", ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /catalog/clients ← 404 sin registros");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// NOMBRE CLIENTE  (C# NomCte compat)
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/clients/{id}/nombre",
    params(("id" = i32, Path, description = "Id del cliente")),
    responses(
        (status = 200, description = "Nombre del cliente",     body = Value),
        (status = 404, description = "Cliente no encontrado",  body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Clients"
)]
pub async fn nombre_cliente(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/clients/{}/nombre", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(c)) => {
            info!("GET /catalog/clients/{}/nombre ← 200 '{}'", id, c.nombre);
            (StatusCode::OK, Json(json!({ "id": c.id, "nombre": c.nombre })))
        }
        Ok(None) => {
            info!("GET /catalog/clients/{}/nombre ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "No existe el cliente" })))
        }
        Err(ret) => {
            error!("GET /catalog/clients/{}/nombre ← 500 codigo={}", id, ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE TIPOS (catálogo tipo 3)
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/clients/tipos",
    responses(
        (status = 200, description = "Lista de tipos",       body = Value),
        (status = 404, description = "Sin registros",        body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Clients"
)]
pub async fn obtiene_tipos(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/clients/tipos");

    match svc::obtiene_tipos(&state.postgres).await {
        Ok(lista) => {
            info!("GET /catalog/clients/tipos ← 200 {} tipos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|t| json!({
                "id":     t.id,
                "nombre": t.nombre,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /catalog/clients/tipos ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /catalog/clients/tipos ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
