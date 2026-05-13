// Programa...: handler::catalog_g::providers
// Descripción: Endpoints HTTP para el catálogo de proveedores
// Origen.....: Proveedores.aspx.cs
//
// Rutas:
//   POST   /catalog/providers           → alta
//   DELETE /catalog/providers/{id}      → baja
//   PUT    /catalog/providers           → cambio
//   GET    /catalog/providers/{id}      → consulta
//   GET    /catalog/providers           → carga_proveedores  (?activos=bool, default true)
//   GET    /catalog/providers/tipos     → obtiene_tipos      (catálogo tipo 3)
//   GET    /catalog/providers/giros     → obtiene_giros      (catálogo tipo 4)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::domain::models::proveedores::Proveedores;
use crate::infrastructure::db::app_state::AppState;
use crate::services::catalog_g::providers as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProveedorInput {
    pub id:           Option<i32>,
    pub nombre:       String,
    pub contacto:     String,
    pub direccion:    String,
    pub telefono:     String,
    pub mail:         String,
    pub cuenta_banco: String,
    pub tipo:         i32,
    pub giro:         i32,
    pub comentarios:  String,
    pub activo:       bool,
    pub rfc:          String,
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
    path = "/catalog/providers",
    request_body = ProveedorInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "Providers"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<ProveedorInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /catalog/providers → nombre='{}' tipo={} giro={}", body.nombre, body.tipo, body.giro);

    let prov = Proveedores {
        id:           body.id,
        nombre:       body.nombre,
        contacto:     body.contacto,
        direccion:    body.direccion,
        telefono:     body.telefono,
        mail:         body.mail,
        cuenta_banco: body.cuenta_banco,
        tipo:         body.tipo,
        tipo_nombre:  None, // poblado por SP en lecturas
        giro:         body.giro,
        giro_nombre:  None,
        comentarios:  body.comentarios,
        activo:       body.activo,
        rfc:          body.rfc,
    };
    let ret = svc::alta(&state.postgres, &prov).await;

    if ret.afectado > 0 {
        info!("POST /catalog/providers ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED,     Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /catalog/providers ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/catalog/providers/{id}",
    params(("id" = i32, Path, description = "Id del proveedor a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "Providers"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /catalog/providers/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /catalog/providers/{} ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /catalog/providers/{} ← 400 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIO
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/catalog/providers",
    request_body = ProveedorInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Providers"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Json(body): Json<ProveedorInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /catalog/providers → id={:?} nombre='{}'", body.id, body.nombre);

    if body.id.is_none() {
        error!("PUT /catalog/providers ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambio" })),
        );
    }
    let prov = Proveedores {
        id:           body.id,
        nombre:       body.nombre,
        contacto:     body.contacto,
        direccion:    body.direccion,
        telefono:     body.telefono,
        mail:         body.mail,
        cuenta_banco: body.cuenta_banco,
        tipo:         body.tipo,
        tipo_nombre:  None,
        giro:         body.giro,
        giro_nombre:  None,
        comentarios:  body.comentarios,
        activo:       body.activo,
        rfc:          body.rfc,
    };
    let ret = svc::cambio(&state.postgres, &prov).await;

    if ret.afectado > 0 {
        info!("PUT /catalog/providers ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /catalog/providers ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/providers/{id}",
    params(("id" = i32, Path, description = "Id del proveedor a consultar")),
    responses(
        (status = 200, description = "Registro encontrado",    body = Value),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Providers"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/providers/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(p)) => {
            info!("GET /catalog/providers/{} ← 200 nombre='{}'", id, p.nombre);
            (StatusCode::OK, Json(json!({
                "id":           p.id,
                "nombre":       p.nombre,
                "contacto":     p.contacto,
                "direccion":    p.direccion,
                "telefono":     p.telefono,
                "mail":         p.mail,
                "cuenta_banco": p.cuenta_banco,
                "tipo":         p.tipo,
                "tipo_nombre":  p.tipo_nombre,
                "giro":         p.giro,
                "giro_nombre":  p.giro_nombre,
                "comentarios":  p.comentarios,
                "activo":       p.activo,
                "rfc":          p.rfc,
            })))
        }
        Ok(None) => {
            info!("GET /catalog/providers/{} ← 404", id);
            (StatusCode::NOT_FOUND,             Json(json!({ "codigo": -41, "mensaje": "No existe el registro" })))
        }
        Err(ret) => {
            error!("GET /catalog/providers/{} ← 500 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CARGA PROVEEDORES
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/providers",
    params(("activos" = Option<bool>, Query, description = "true = sólo activos (default), false = todos")),
    responses(
        (status = 200, description = "Lista de proveedores",  body = Value),
        (status = 404, description = "Sin registros",         body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Providers"
)]
pub async fn carga_proveedores(
    State(state): State<AppState>,
    Query(filtro): Query<FiltroActivos>,
) -> (StatusCode, Json<Value>) {
    let activos = filtro.activos.unwrap_or(true);
    debug!("GET /catalog/providers?activos={}", activos);

    match svc::carga_proveedores(&state.postgres, activos).await {
        Ok(lista) => {
            info!("GET /catalog/providers ← 200 {} registros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|p| json!({
                "id":          p.id,
                "nombre":      p.nombre,
                "rfc":         p.rfc,
                "contacto":    p.contacto,
                "telefono":    p.telefono,
                "tipo":        p.tipo,
                "tipo_nombre": p.tipo_nombre,
                "giro":        p.giro,
                "giro_nombre": p.giro_nombre,
                "activo":      p.activo,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /catalog/providers ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /catalog/providers ← 404 sin registros");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE TIPOS (catálogo tipo 3)
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/providers/tipos",
    responses(
        (status = 200, description = "Lista de tipos",        body = Value),
        (status = 404, description = "Sin registros",         body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Providers"
)]
pub async fn obtiene_tipos(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/providers/tipos");

    match svc::obtiene_tipos(&state.postgres).await {
        Ok(lista) => {
            info!("GET /catalog/providers/tipos ← 200 {} tipos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|t| json!({
                "id":     t.id,
                "nombre": t.nombre,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /catalog/providers/tipos ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /catalog/providers/tipos ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE GIROS (catálogo tipo 4)
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/catalog/providers/giros",
    responses(
        (status = 200, description = "Lista de giros",        body = Value),
        (status = 404, description = "Sin registros",         body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "Providers"
)]
pub async fn obtiene_giros(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /catalog/providers/giros");

    match svc::obtiene_giros(&state.postgres).await {
        Ok(lista) => {
            info!("GET /catalog/providers/giros ← 200 {} giros", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|t| json!({
                "id":     t.id,
                "nombre": t.nombre,
            })).collect::<Vec<_>>())))
        }
        Err(ret) if ret.codigo < -50 => {
            error!("GET /catalog/providers/giros ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            info!("GET /catalog/providers/giros ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}
