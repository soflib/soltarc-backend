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
//   PUT    /operaciones/proyectos/{id}/grupo-usuario     → gpo_usr_proy (lista completa de asignaciones)
//   GET    /operaciones/proyectos/{id}/asignaciones      → get_asignaciones
//   GET    /operaciones/proyectos/{id}/cliente           → cliente_proy
//   GET    /operaciones/proyectos/{id}/directorio        → dir_proy
//   GET    /operaciones/proyectos/{id}/total-ppto        → total_ppto

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension,
    Json,
};
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use utoipa::ToSchema;
use tracing::{debug, error, info};

use crate::api::middleware::roles::AuthUser;
use crate::domain::models::lookup::LookupItem;
use crate::domain::models::proyectos::Proyectos;
use crate::generated::auth::GetAllUsersRequest;
use crate::infrastructure::db::app_state::AppState;
use crate::services::operaciones::proyectos as svc;

#[derive(Debug, Deserialize)]
pub struct ProyLookupQuery {
    pub q:       Option<String>,
    pub cliente: Option<i32>,   // filtro opcional: solo proyectos de este cliente
    pub limit:   Option<i32>,
}

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

/// Un par (grupo, usuario) asignado al proyecto. usuario 0 = "todo el grupo".
#[derive(Debug, Deserialize, ToSchema)]
pub struct AsignacionInput {
    pub grupo:   i32,
    pub usuario: i32,
}

/// Lista COMPLETA de asignaciones del proyecto (reemplaza las existentes).
/// Lista vacía = quitar todas (solo Admin/nivel 1 verá el proyecto).
#[derive(Debug, Deserialize, ToSchema)]
pub struct AsignacionesInput {
    pub asignaciones: Vec<AsignacionInput>,
}

/// Clonar un proyecto: `nombre` nuevo (obligatorio); `cliente` opcional
/// (None = se mantiene el cliente del proyecto original).
#[derive(Debug, Deserialize, ToSchema)]
pub struct ClonarInput {
    pub nombre:  String,
    pub cliente: Option<i32>,
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
        tenant_id:    None, // lo fija el SP a partir del p_tenant_id
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
    Extension(auth_user): Extension<AuthUser>,
    Json(body): Json<ProyectosInput>,
) -> (StatusCode, Json<Value>) {
    info!("POST /operaciones/proyectos → nombre='{}'", body.nombre);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let proy = match parse_input(body) {
        Ok(p) => p,
        Err(msg) => {
            error!("POST /operaciones/proyectos ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::alta(&state.postgres, &proy, tenant_id).await;

    if ret.afectado > 0 {
        info!("POST /operaciones/proyectos ← 201 afectado={}", ret.afectado);
        (StatusCode::CREATED,     Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else if ret.codigo == -20 {
        // Tope de proyectos del plan alcanzado → 409 (no es error de captura).
        info!("POST /operaciones/proyectos ← 409 límite de plan");
        (StatusCode::CONFLICT,    Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    } else {
        error!("POST /operaciones/proyectos ← 400 codigo={} msg='{}'", ret.codigo, ret.mensaje);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// CLONAR — copia un proyecto completo (header + árbol WBS + asignaciones)
// ─────────────────────────────────────────────
#[utoipa::path(
    post,
    path = "/operaciones/proyectos/{id}/clonar",
    params(("id" = i32, Path, description = "Id del proyecto a clonar")),
    request_body = ClonarInput,
    responses(
        (status = 201, description = "Proyecto clonado",               body = Value),
        (status = 400, description = "Datos inválidos o error BD",     body = Value),
        (status = 409, description = "Tope de proyectos del plan",     body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn clonar(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(body): Json<ClonarInput>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };
    let nombre = body.nombre.trim();
    if nombre.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "mensaje": "El nombre del proyecto clonado es obligatorio." })));
    }
    info!("POST /operaciones/proyectos/{}/clonar → nombre='{}'", id, nombre);

    match svc::clonar(&state.postgres, id, nombre, body.cliente, tenant_id).await {
        Ok(nuevo_id) => {
            info!("POST /operaciones/proyectos/{}/clonar ← 201 nuevo={}", id, nuevo_id);
            (StatusCode::CREATED, Json(json!({ "afectado": nuevo_id, "mensaje": "Proyecto clonado correctamente." })))
        }
        Err(ret) if ret.codigo == -20 => {
            info!("POST /operaciones/proyectos/{}/clonar ← 409 límite de plan", id);
            (StatusCode::CONFLICT, Json(json!({ "codigo": -20, "mensaje": ret.mensaje })))
        }
        Err(ret) => {
            error!("POST /operaciones/proyectos/{}/clonar ← 400 codigo={}", id, ret.codigo);
            (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
    }
}

// ─────────────────────────────────────────────
// CUPO — proyectos usados vs límite del plan (tenant del JWT)
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/cupo",
    responses(
        (status = 200, description = "Proyectos usados vs límite del plan", body = Value),
        (status = 500, description = "Error de base de datos",              body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn cupo(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };
    match svc::cupo(&state.postgres, tenant_id).await {
        Ok((usados, max)) => (StatusCode::OK, Json(json!({
            "usados":        usados,
            "max_proyectos": max,            // null = ilimitado
            "ilimitado":     max.is_none(),
        }))),
        Err(ret) => {
            error!("GET /operaciones/proyectos/cupo ← 500 codigo={}", ret.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
        }
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
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    info!("DELETE /operaciones/proyectos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let ret = svc::baja(&state.postgres, id, tenant_id).await;

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
    Extension(auth_user): Extension<AuthUser>,
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

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let proy = match parse_input(body) {
        Ok(p) => p,
        Err(msg) => {
            error!("PUT /operaciones/proyectos ← 400 parse: {}", msg);
            return (StatusCode::BAD_REQUEST, Json(json!({ "codigo": -1, "mensaje": msg })));
        }
    };

    let ret = svc::cambio(&state.postgres, &proy, tenant_id).await;

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
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/proyectos/{}", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    match svc::consulta(&state.postgres, id, tenant_id).await {
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
    Extension(auth_user): Extension<AuthUser>,
    Query(filtro): Query<FiltroActivos>,
) -> (StatusCode, Json<Value>) {
    let activos = filtro.activos.unwrap_or(true);
    debug!("GET /operaciones/proyectos?activos={}", activos);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    // Control de acceso por NIVEL: Admin ve todo (nivel 1); el resto según
    // su perfil de negocio (1=todo, 2=su grupo, 3=solo lo asignado a él).
    // Sin perfil → nivel 1, para no bloquear usuarios aún no configurados.
    // El filtrado lo hace el SP contra cpa_proyecto_asignaciones.
    let (grupo, gn_usr_id, nivel) = if auth_user.role.eq_ignore_ascii_case("Admin") {
        (0, 0, 1)
    } else {
        match uuid::Uuid::parse_str(&auth_user.user_id) {
            Ok(uid) => crate::dal::gn_usuarios::perfil_de(&state.postgres, tenant_id, uid).await,
            Err(_)  => (0, 0, 1),
        }
    };

    match svc::llena_proyectos(&state.postgres, activos, tenant_id, grupo, gn_usr_id, nivel).await {
        Ok(visibles) => {
            info!("GET /operaciones/proyectos ← 200 {} proyectos (nivel {})", visibles.len(), nivel);
            (StatusCode::OK, Json(json!(visibles.iter().map(|p| json!({
                "id":          p.id,
                "tipo":        p.tipo,
                "nombre":      p.nombre,
                "descripcion": p.descripcion,
                "direccion":   p.direccion,
                "estado":      p.estado,
                "presupuesto": p.presupuesto.to_string(),
                "fecha_ini":   p.fecha_ini.format("%Y-%m-%d").to_string(),
                "fecha_fin":   p.fecha_fin.format("%Y-%m-%d").to_string(),
                "asignado":    p.asignado,
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
// GPO USR PROYECTO — reemplaza la lista completa de asignaciones
// ─────────────────────────────────────────────
#[utoipa::path(
    put,
    path = "/operaciones/proyectos/{id}/grupo-usuario",
    params(("id" = i32, Path, description = "Id del proyecto")),
    request_body = AsignacionesInput,
    responses(
        (status = 200, description = "Actualización realizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error BD", body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn gpo_usr_proy(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
    Json(body): Json<AsignacionesInput>,
) -> (StatusCode, Json<Value>) {
    info!("PUT /operaciones/proyectos/{}/grupo-usuario → {} asignaciones", id, body.asignaciones.len());

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let grupos:   Vec<i32> = body.asignaciones.iter().map(|a| a.grupo).collect();
    let usuarios: Vec<i32> = body.asignaciones.iter().map(|a| a.usuario).collect();

    let ret = svc::asignaciones_set(&state.postgres, id, tenant_id, &grupos, &usuarios).await;

    if ret.codigo == 30 {
        info!("PUT /operaciones/proyectos/{}/grupo-usuario ← 200 {} pares", id, ret.afectado);
        (StatusCode::OK,          Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /operaciones/proyectos/{}/grupo-usuario ← 400 codigo={}", id, ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ─────────────────────────────────────────────
// ASIGNACIONES DEL PROYECTO
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/{id}/asignaciones",
    params(("id" = i32, Path, description = "Id del proyecto")),
    responses(
        (status = 200, description = "Lista de asignaciones (puede ser vacía)", body = Value),
        (status = 500, description = "Error de base de datos",                  body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn get_asignaciones(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /operaciones/proyectos/{}/asignaciones", id);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    let lista = match svc::asignaciones_lst(&state.postgres, id, tenant_id).await {
        Ok(l)  => l,
        Err(ret) => {
            error!("GET /operaciones/proyectos/{}/asignaciones ← 500 codigo={}", id, ret.codigo);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })));
        }
    };

    // Resolver user_id (email) al NOMBRE real vía gRPC de auth — mismo patrón
    // que usuarios_grupo. Sin asignaciones con usuario, ahorra la llamada.
    let mapa: std::collections::HashMap<String, String> = if lista.iter().any(|a| a.gn_usr_id > 0) {
        let mut client = state.auth_grpc.clone();
        let req = GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: auth_user.tenant_id.clone() };
        match client.get_all_users(req).await {
            Ok(r) => r.users.into_iter().map(|u| {
                let nombre = if !u.full_name.trim().is_empty() { u.full_name }
                             else if !u.username.trim().is_empty() { u.username }
                             else { u.email.clone() };
                (u.email, nombre)
            }).collect(),
            Err(_) => std::collections::HashMap::new(),
        }
    } else {
        std::collections::HashMap::new()
    };

    let items: Vec<Value> = lista.iter().map(|a| json!({
        "grupo":          a.gn_id,
        "usuario":        a.gn_usr_id,
        "grupo_nombre":   a.grupo_nombre,
        "usuario_nombre": if a.gn_usr_id > 0 {
            mapa.get(&a.usuario_user_id).cloned().unwrap_or_else(|| a.usuario_user_id.clone())
        } else {
            String::new()
        },
    })).collect();

    info!("GET /operaciones/proyectos/{}/asignaciones ← 200 {} pares", id, items.len());
    (StatusCode::OK, Json(json!(items)))
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
    Extension(auth_user): Extension<AuthUser>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };
    match svc::lista_grupos(&state.postgres, tenant_id).await {
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
    Extension(auth_user): Extension<AuthUser>,
    Path(grupo_id): Path<i32>,
) -> (StatusCode, Json<Value>) {
    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(_) => return (StatusCode::OK, Json(json!([]))),
    };

    let lista = match svc::usuarios_grupo(&state.postgres, grupo_id, tenant_id).await {
        Ok(l)  => l,
        Err(_) => return (StatusCode::OK, Json(json!([]))),
    };

    // Resolver el user_id (email del seed) al NOMBRE real del usuario, vía gRPC
    // de auth. gn_usuarios.user_id == auth.users.email; mostramos full_name.
    let mut client = state.auth_grpc.clone();
    let req = GetAllUsersRequest { limit: 1000, offset: 0, tenant_id: auth_user.tenant_id.clone() };
    let mapa: std::collections::HashMap<String, String> = match client.get_all_users(req).await {
        Ok(r) => r.users.into_iter().map(|u| {
            let nombre = if !u.full_name.trim().is_empty() { u.full_name }
                         else if !u.username.trim().is_empty() { u.username }
                         else { u.email.clone() };
            (u.email, nombre)
        }).collect(),
        Err(_) => std::collections::HashMap::new(),
    };

    let items: Vec<Value> = lista.iter().map(|u| json!({
        "id":     u.id,
        "nombre": mapa.get(&u.user_id).cloned().unwrap_or_else(|| u.user_id.clone()),
    })).collect();
    (StatusCode::OK, Json(json!(items)))
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

// ─────────────────────────────────────────────
// LOOKUP — autocomplete proyectos activos
// Etiqueta: "<nombre proyecto> — <cliente>"
// GET /operaciones/proyectos/lookup?q=foo&limit=20
// ─────────────────────────────────────────────
#[utoipa::path(
    get,
    path = "/operaciones/proyectos/lookup",
    params(
        ("q"     = Option<String>, Query, description = "Texto a buscar (ILIKE — busca en nombre del proyecto y nombre del cliente)"),
        ("limit" = Option<i32>,    Query, description = "Máximo de resultados (default 20, máx 100)"),
    ),
    responses(
        (status = 200, description = "Lista [{id, etiqueta}]",     body = Value),
        (status = 500, description = "Error de base de datos",     body = Value),
    ),
    tag = "Proyectos"
)]
pub async fn lookup(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(q): Query<ProyLookupQuery>,
) -> (StatusCode, Json<Value>) {
    let qs    = q.q.unwrap_or_default();
    let limit = q.limit.unwrap_or(20).clamp(1, 100);
    debug!("GET /operaciones/proyectos/lookup q='{}' cliente={:?} limit={}", qs, q.cliente, limit);

    let tenant_id = match auth_user.tenant_uuid() {
        Ok(t) => t,
        Err(e) => return e,
    };

    // Scopeado por perfil: Admin/Finanzas (nivel 1) ven todo; Arquitecto (2) solo
    // su grupo; Reportes (3) solo lo asignado. Consistente con la lista de proyectos.
    let (grupo, gn_usr_id, nivel) =
        crate::dal::gn_usuarios::perfil_de_auth(&state.postgres, tenant_id, &auth_user.user_id, &auth_user.role).await;
    let items: Vec<LookupItem> =
        crate::dal::proyectos::lookup_accesibles(&state.postgres, tenant_id, grupo, gn_usr_id, nivel, &qs, q.cliente, limit).await;
    info!("GET /operaciones/proyectos/lookup ← 200 {} items (nivel {})", items.len(), nivel);
    (StatusCode::OK, Json(json!(items)))
}
