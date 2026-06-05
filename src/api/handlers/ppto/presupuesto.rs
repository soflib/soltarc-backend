// Rutas:
//   POST   /ppto/presupuestos        → alta
//   DELETE /ppto/presupuestos/{id}   → baja
//   PUT    /ppto/presupuestos        → cambio
//   GET    /ppto/presupuestos/{id}   → consulta
//   GET    /ppto/presupuestos        → carga_pptos (?gpo_neg=&gpo_user_id=&usr_nivel=&activos=)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::lookup::LookupItem;
use crate::domain::models::presupuesto::Presupuesto;
use crate::infrastructure::db::app_state::AppState;
use crate::services::ppto::presupuesto as svc;

#[derive(Debug, Deserialize)]
pub struct PptoLookupQuery {
    pub q:       Option<String>,
    pub cliente: Option<i32>,
    pub limit:   Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PresupuestoInput {
    pub id:          Option<i32>,
    pub nombre:      String,
    pub descripcion: String,
    pub direccion:   String,
    pub comentarios: String,
    pub fecha:       String,
    pub cliente:     i32,
    pub activo:      bool,
    pub estado:      i32,
    pub pie_pagina:  String,
    pub gn_id:       i32,
    pub gn_user_id:  i32,
}

#[derive(Debug, Deserialize)]
pub struct PresupuestosQuery {
    pub gpo_neg:     Option<i32>,
    pub gpo_user_id: Option<i32>,
    pub usr_nivel:   Option<i32>,
    pub activos:     Option<bool>,
}

fn input_to_model(body: PresupuestoInput) -> Presupuesto {
    Presupuesto {
        id:          body.id,
        nombre:      body.nombre,
        descripcion: body.descripcion,
        direccion:   body.direccion,
        comentarios: body.comentarios,
        fecha:       body.fecha,
        cliente:     body.cliente,
        activo:      body.activo,
        estado:      body.estado,
        pie_pagina:  body.pie_pagina,
        gn_id:       body.gn_id,
        gn_user_id:  body.gn_user_id,
    }
}

fn ppto_json(p: &Presupuesto) -> Value {
    json!({
        "id":          p.id,
        "nombre":      p.nombre,
        "descripcion": p.descripcion,
        "direccion":   p.direccion,
        "comentarios": p.comentarios,
        "fecha":       p.fecha,
        "cliente":     p.cliente,
        "activo":      p.activo,
        "estado":      p.estado,
        "pie_pagina":  p.pie_pagina,
        "gn_id":       p.gn_id,
        "gn_user_id":  p.gn_user_id,
    })
}

// ── Alta ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/ppto/presupuestos",
    request_body = PresupuestoInput,
    responses(
        (status = 201, description = "Presupuesto registrado",   body = Value),
        (status = 400, description = "Alta cancelada o error",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn alta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<PresupuestoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(nombre = %body.nombre, "POST /ppto/presupuestos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ppto = input_to_model(body);
    let ret = svc::alta(&state.postgres, &ppto, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /ppto/presupuestos ← 201 id={}", ret.afectado);
        (StatusCode::CREATED, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("POST /ppto/presupuestos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Baja ──────────────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/ppto/presupuestos/{id}",
    params(("id" = i32, Path, description = "Id del presupuesto a eliminar")),
    responses(
        (status = 200, description = "Presupuesto eliminado",    body = Value),
        (status = 400, description = "Baja cancelada o error",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn baja(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /ppto/presupuestos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

    if ret.afectado > 0 {
        info!("DELETE /ppto/presupuestos/{} ← 200 OK", id);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("DELETE /ppto/presupuestos/{} ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Cambio ────────────────────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/ppto/presupuestos",
    request_body = PresupuestoInput,
    responses(
        (status = 200, description = "Presupuesto actualizado",               body = Value),
        (status = 400, description = "Actualización cancelada o error",       body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn cambio(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<PresupuestoInput>,
) -> (StatusCode, Json<Value>) {
    debug!(id = ?body.id, "PUT /ppto/presupuestos");

    let Some(_) = body.id else {
        return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": "El campo id es requerido para cambio" })));
    };

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ppto = input_to_model(body);
    let ret = svc::cambio(&state.postgres, &ppto, tenant_id).await;

    if ret.afectado > 0 {
        info!("PUT /ppto/presupuestos ← 200 OK afectado={}", ret.afectado);
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /ppto/presupuestos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Consulta ──────────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/presupuestos/{id}",
    params(("id" = i32, Path, description = "Id del presupuesto a consultar")),
    responses(
        (status = 200, description = "Presupuesto encontrado",   body = Value),
        (status = 404, description = "Presupuesto no encontrado", body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn consulta(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /ppto/presupuestos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
        Ok(Some(p)) => {
            info!("GET /ppto/presupuestos/{} ← 200", id);
            (StatusCode::OK, Json(ppto_json(&p)))
        }
        Ok(None) => {
            info!("GET /ppto/presupuestos/{} ← 404", id);
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Presupuesto no encontrado" })))
        }
        Err(rc) => {
            error!("GET /ppto/presupuestos/{} ← 500 codigo={}", id, rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ── Carga Presupuestos ────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/ppto/presupuestos",
    params(
        ("gpo_neg"     = i32,         Query, description = "Id del grupo de negocio"),
        ("gpo_user_id" = i32,         Query, description = "Id de usuario en el grupo"),
        ("usr_nivel"   = i32,         Query, description = "Nivel del usuario"),
        ("activos"     = Option<bool>, Query, description = "true = solo activos, false = todos"),
    ),
    responses(
        (status = 200, description = "Lista de presupuestos",    body = Value),
        (status = 400, description = "Faltan parámetros",        body = Value),
        (status = 404, description = "Sin presupuestos",         body = Value),
        (status = 500, description = "Error de base de datos",   body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn carga_pptos(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<PresupuestosQuery>,
) -> (StatusCode, Json<Value>) {
    // El aislamiento por tenant ya lo aplica ppto_sp_presupuesto_lstact. Los
    // parámetros de grupo son opcionales: por defecto usr_nivel=1 → el usuario
    // ve TODOS los presupuestos de su tenant (mismo patrón que proyectos/clientes,
    // que sólo reciben `activos`). El frontend manda `?proyecto=` y se ignora.
    let gpo_neg     = q.gpo_neg.unwrap_or(0);
    let gpo_user_id = q.gpo_user_id.unwrap_or(0);
    let usr_nivel   = q.usr_nivel.unwrap_or(1);
    let activos     = q.activos.unwrap_or(true);

    debug!(gpo_neg, gpo_user_id, usr_nivel, activos, "GET /ppto/presupuestos");

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::carga_pptos(&state.postgres, gpo_neg, gpo_user_id, usr_nivel, activos, tenant_id).await {
        Ok(lista) => {
            info!("GET /ppto/presupuestos ← 200 {} registros", lista.len());
            let items: Vec<Value> = lista.iter().map(ppto_json).collect();
            (StatusCode::OK, Json(json!({ "presupuestos": items, "total": items.len() })))
        }
        Err(rc) if rc.codigo > -85 => {
            info!("GET /ppto/presupuestos ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
        Err(rc) => {
            error!("GET /ppto/presupuestos ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// LOOKUP — autocomplete presupuestos activos
// Filtro opcional por cliente.
// GET /ppto/presupuestos/lookup?q=foo&cliente=17&limit=20
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/ppto/presupuestos/lookup",
    params(
        ("q"       = Option<String>, Query, description = "Texto a buscar (ILIKE)"),
        ("cliente" = Option<i32>,    Query, description = "Filtra por cliente (omitir = todos)"),
        ("limit"   = Option<i32>,    Query, description = "Máximo de resultados (default 20, máx 100)"),
    ),
    responses(
        (status = 200, description = "Lista [{id, etiqueta}]",     body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "PptoPresupuestos"
)]
pub async fn lookup(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<PptoLookupQuery>,
) -> (StatusCode, Json<Value>) {
    let qs    = q.q.unwrap_or_default();
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    debug!("GET /ppto/presupuestos/lookup q='{}' cliente={:?} limit={}", qs, q.cliente, limit);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::lookup(&state.postgres, &qs, q.cliente, limit, tenant_id).await {
        Ok(items) => {
            info!("GET /ppto/presupuestos/lookup ← 200 {} items", items.len());
            let payload: Vec<LookupItem> = items;
            (StatusCode::OK, Json(json!(payload)))
        }
        Err(rc) => {
            error!("GET /ppto/presupuestos/lookup ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
