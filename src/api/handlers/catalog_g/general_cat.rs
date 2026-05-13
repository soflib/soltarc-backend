// Programa...: handler::catalogs::cat_general
// Descripción: Endpoints HTTP para el catálogo general
// Origen.....: CatGeneral.aspx.cs
//
// Rutas:
//   POST   /general/catalogs              → alta
//   DELETE /general/catalogs/{id}         → baja
//   PUT    /general/catalogs              → cambios
//   GET    /general/catalogs/{id}         → consulta
//   GET    /general/catalogs              → obtiene_todo
//   GET    /general/catalogs/tipo/{tipo}  → obtiene_por_tipo
//   GET    /general/catalog-types         → obtiene_tipos

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, error, info};

use crate::infrastructure::db::app_state::AppState;
use crate::services::catalog_g::general_cat as svc;
use crate::domain::models::catalog_g::{
    CatalogGInput,
    CatalogGOutput,
};

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/general/catalogs",
    request_body = CatalogGInput,
    responses(
        (status = 201, description = "Alta realizada",            body = Value),
        (status = 400, description = "Alta cancelada o error BD", body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn alta(
    State(state): State<AppState>,
    Json(body): Json<CatalogGInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /general/catalogs → tipo={:?} nombre='{}'", body.tipo, body.nombre);

    let ret = svc::alta(&state.postgres, &body).await;

    if ret.afectado > 0 {
        info!("POST /general/catalogs ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED,     Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /general/catalogs ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
#[utoipa::path(
    delete,
    path = "/general/catalogs/{id}",
    params(("id" = i32, Path, description = "Id del registro a eliminar")),
    responses(
        (status = 200, description = "Baja realizada",            body = Value),
        (status = 400, description = "Baja cancelada o error BD", body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn baja(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /general/catalogs/{}", id);

    let ret = svc::baja(&state.postgres, id).await;

    if ret.afectado > 0 {
        info!("DELETE /general/catalogs/{} ← 200 OK", id);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /general/catalogs/{} ← 400 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/general/catalogs",
    request_body = CatalogGInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn cambios(
    State(state): State<AppState>,
    Json(body): Json<CatalogGInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /general/catalogs → id={:?} nombre='{}'", body.id, body.nombre);

    let Some(_id) = body.id else {
        error!("PUT /general/catalogs ← 400 falta id");
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambios" })),
        );
    };

    let ret = svc::cambios(&state.postgres, &body).await;

    if ret.afectado > 0 {
        info!("PUT /general/catalogs ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /general/catalogs ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/general/catalogs/{id}",
    params(("id" = i32, Path, description = "Id del registro a consultar")),
    responses(
        (status = 200, description = "Registro encontrado",    body = CatalogGOutput),
        (status = 404, description = "Registro no encontrado", body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /general/catalogs/{}", id);

    match svc::consulta(&state.postgres, id).await {
        Ok(Some(cat)) => {
            info!("GET /general/catalogs/{} ← 200 nombre={:?}", id, cat.nombre);
            (
                StatusCode::OK,
                Json(json!({
                    "id":          cat.id,
                    "tipo":        cat.tipo,
                    "nombre":      cat.nombre,
                    "activo":      cat.activo,
                    "comentarios": cat.comentarios,
                })),
            )
        }
        Ok(None) => {
            info!("GET /general/catalogs/{} ← 404", id);
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "codigo": -41, "mensaje": "No existe el registro" })),
            )
        }
        Err(ret) => {
            error!("GET /general/catalogs/{} ← 500 codigo={} msg='{}'", id, ret.codigo, ret.mensaje);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })),
            )
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE TODO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/general/catalogs",
    responses(
        (status = 200, description = "Lista del catálogo",  body = Vec<CatalogGOutput>),
        (status = 404, description = "Sin registros",       body = Value),
        (status = 500, description = "Error de base de datos", body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn obtiene_todo(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /general/catalogs");

    match svc::obtiene_todo(&state.postgres).await {
        Ok(lista) => {
            info!("GET /general/catalogs ← 200 {} registros", lista.len());
            (
                StatusCode::OK,
                Json(json!(lista
                    .iter()
                    .map(|c| json!({
                        "id":          c.id,
                        "tipo":        c.tipo,
                        "nombre":      c.nombre,
                        "activo":      c.activo,
                        "comentarios": c.comentarios,
                    }))
                    .collect::<Vec<_>>())),
            )
        }
        Err(ret) if ret.codigo == -31 => {
            info!("GET /general/catalogs ← 404 sin registros");
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })),
            )
        }
        Err(ret) => {
            error!("GET /general/catalogs ← 500 codigo={} msg='{}'", ret.codigo, ret.mensaje);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })),
            )
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE POR TIPO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/general/catalogs/tipo/{tipo}",
    params(("tipo" = i32, Path, description = "Código de tipo: 0=Sistema 1=Estado proy 2=Tipo proy 3=Pers.moral 4=Tipo prov 5=Bancos 6=Tipo Tarea 7=Estado PPTO 8=Estado Partidas")),
    responses(
        (status = 200, description = "Catálogos del tipo indicado", body = Vec<CatalogGOutput>),
        (status = 404, description = "Sin registros para ese tipo", body = Value),
        (status = 500, description = "Error de base de datos",      body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn obtiene_por_tipo(
    State(state): State<AppState>,
    Path(tipo): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /general/catalogs/tipo/{}", tipo);

    match svc::obtiene_por_tipo(&state.postgres, tipo).await {
        Ok(lista) => {
            info!("GET /general/catalogs/tipo/{} ← 200 {} registros", tipo, lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|c| json!({
                "id":          c.id,
                "tipo":        c.tipo,
                "nombre":      c.nombre,
                "activo":      c.activo,
                "comentarios": c.comentarios,
            })).collect::<Vec<_>>())))
        }
        Err(rc) if rc.codigo > -35 => {
            info!("GET /general/catalogs/tipo/{} ← 404", tipo);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /general/catalogs/tipo/{} ← 500 codigo={}", tipo, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE TIPOS
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/general/catalog-types",
    responses(
        (status = 200, description = "Tipos de catálogo disponibles", body = Vec<CatalogGOutput>),
        (status = 404, description = "Sin tipos registrados",         body = Value),
        (status = 500, description = "Error de base de datos",        body = Value),
    ),
    tag = "General Catalogs"
)]
pub async fn obtiene_tipos(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /general/catalog-types");

    match svc::obtiene_tipos(&state.postgres).await {
        Ok(lista) => {
            info!("GET /general/catalog-types ← 200 {} tipos", lista.len());
            (StatusCode::OK, Json(json!(lista.iter().map(|c| json!({
                "id":          c.id,
                "tipo":        c.tipo,
                "nombre":      c.nombre,
                "activo":      c.activo,
                "comentarios": c.comentarios,
            })).collect::<Vec<_>>())))
        }
        Err(rc) if rc.codigo > -55 => {
            info!("GET /general/catalog-types ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /general/catalog-types ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
