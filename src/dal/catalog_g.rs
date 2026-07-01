// Programa...: catalog_g
// Descripción: Operaciones de la tabla cpa_Catalogos (multi-tenant)
// Origen.....: oCatalogoG.cs
//
// Stored Procedures que usa:
//   sp_cpa_catalogo_add       → alta            (tipo, nombre, activo, comentarios, tenant_id)
//   sp_cpa_catalogo_del       → baja            (id, tenant_id)
//   sp_cpa_catalogo_upd       → cambios         (id, tipo, nombre, activo, comentarios, tenant_id)
//   sp_cpa_catalogo_qry       → consulta por id (id, tenant_id)
//   sp_cpa_catalogo_lst_all   → lista todos     (tenant_id)
//   sp_cpa_catalogo_qry_tipo  → lista por tipo  (tipo, tenant_id)
//   sp_cpa_catalogo_lst_tipos → lista tipos     (tenant_id)
//   sp_cpa_catalogo_lookup    → autocomplete    (tipo, q, limit, tenant_id)
//
// Todos los SPs filtran por (tenant_id IS NULL OR tenant_id = p_tenant_id):
//   - tenant ve catálogos globales (NULL) + sus propios
//   - sp_del / sp_upd solo afectan filas del propio tenant (globales protegidos)

use crate::domain::models::catalog_g::CatalogG;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use tracing::{debug, error, info};
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_catalogo_add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cat: &CatalogG, tenant_id: Uuid) -> ReturnCode {
    debug!("dal::catalog_g::alta → tipo={:?} nombre={:?} activo={:?} tenant={}",
           cat.tipo, cat.nombre, cat.activo, tenant_id);

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_catalogo_add($1, $2, $3, $4, $5)"
    )
    .bind(cat.tipo)
    .bind(&cat.nombre)
    .bind(cat.activo)
    .bind(&cat.comentarios)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => {
            info!("dal::catalog_g::alta ← OK id={}", id);
            ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() }
        }
        Ok(n) => {
            error!("dal::catalog_g::alta ← cancelada, sp retornó {}", n);
            ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() }
        }
        Err(e) => {
            error!("dal::catalog_g::alta ← DB error: {}", e);
            ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() }
        }
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_catalogo_del
// Solo borra si el registro pertenece al tenant.
// Globales (tenant_id IS NULL) nunca se borran → devuelve afectado=0.
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    debug!("dal::catalog_g::baja → id={} tenant={}", id, tenant_id);

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_catalogo_del($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => {
            info!("dal::catalog_g::baja ← OK afectado={}", n);
            ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() }
        }
        Ok(n) => {
            error!("dal::catalog_g::baja ← cancelada (¿global o de otro tenant?) sp retornó {}", n);
            ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada (registro no encontrado o no autorizado)".to_string() }
        }
        Err(e) => {
            error!("dal::catalog_g::baja ← DB error: {}", e);
            ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }
        }
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_cpa_catalogo_upd
// Solo actualiza si el registro pertenece al tenant.
// Globales no se editan → devuelve afectado=0.
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cat: &CatalogG, tenant_id: Uuid) -> ReturnCode {
    let id = cat.id.unwrap_or(0);
    debug!("dal::catalog_g::cambios → id={} tipo={:?} nombre={:?} tenant={}",
           id, cat.tipo, cat.nombre, tenant_id);

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_catalogo_upd($1, $2, $3, $4, $5, $6)"
    )
    .bind(id)
    .bind(cat.tipo)
    .bind(&cat.nombre)
    .bind(cat.activo)
    .bind(&cat.comentarios)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => {
            info!("dal::catalog_g::cambios ← OK afectado={}", n);
            ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() }
        }
        Ok(n) => {
            error!("dal::catalog_g::cambios ← cancelada (¿global o de otro tenant?) sp retornó {}", n);
            ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada (registro no encontrado o no autorizado)".to_string() }
        }
        Err(e) => {
            error!("dal::catalog_g::cambios ← DB error: {}", e);
            ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() }
        }
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_catalogo_qry
// Devuelve el registro si es global o pertenece al tenant.
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::consulta → id={} tenant={}", id, tenant_id);

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM soltarc.sp_cpa_catalogo_qry($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(r)) => {
            info!("dal::catalog_g::consulta ← encontrado nombre={:?}", r.nombre);
            Ok(Some(r))
        }
        Ok(None) => {
            info!("dal::catalog_g::consulta ← no existe id={} (o no visible para tenant)", id);
            Ok(None)
        }
        Err(e) => {
            error!("dal::catalog_g::consulta ← DB error: {}", e);
            Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() })
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE TODO — sp_cpa_catalogo_lst_all
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::obtiene_todo → tenant={}", tenant_id);

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM soltarc.sp_cpa_catalogo_lst_all($1)"
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => {
            info!("dal::catalog_g::obtiene_todo ← {} registros", lista.len());
            Ok(lista)
        }
        Ok(_) => {
            info!("dal::catalog_g::obtiene_todo ← lista vacía");
            Err(ReturnCode { codigo: -31, afectado: 0, mensaje: "No hay entradas".to_string() })
        }
        Err(e) => {
            error!("dal::catalog_g::obtiene_todo ← DB error: {}", e);
            Err(ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() })
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE POR TIPO — sp_cpa_catalogo_qry_tipo
// ─────────────────────────────────────────────
pub async fn obtiene_por_tipo(pool: &PgPool, tipo: i32, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::obtiene_por_tipo → tipo={} tenant={}", tipo, tenant_id);

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM soltarc.sp_cpa_catalogo_qry_tipo($1, $2)"
    )
    .bind(tipo as i16)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => {
            info!("dal::catalog_g::obtiene_por_tipo ← {} registros tipo={}", lista.len(), tipo);
            Ok(lista)
        }
        Ok(_) => {
            info!("dal::catalog_g::obtiene_por_tipo ← vacío tipo={}", tipo);
            Err(ReturnCode { codigo: -30, afectado: 0, mensaje: "No hay entradas al catálogo".to_string() })
        }
        Err(e) => {
            error!("dal::catalog_g::obtiene_por_tipo ← DB error tipo={}: {}", tipo, e);
            Err(ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() })
        }
    }
}

// ─────────────────────────────────────────────
// OBTIENE TIPOS — sp_cpa_catalogo_lst_tipos
// ─────────────────────────────────────────────
pub async fn obtiene_tipos(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::obtiene_tipos → tenant={}", tenant_id);

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM soltarc.sp_cpa_catalogo_lst_tipos($1)"
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => {
            info!("dal::catalog_g::obtiene_tipos ← {} tipos", lista.len());
            Ok(lista)
        }
        Ok(_) => {
            info!("dal::catalog_g::obtiene_tipos ← sin tipos");
            Err(ReturnCode { codigo: -51, afectado: 0, mensaje: "No hay Tipos en el sistema".to_string() })
        }
        Err(e) => {
            error!("dal::catalog_g::obtiene_tipos ← DB error: {}", e);
            Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() })
        }
    }
}

// ─────────────────────────────────────────────
// LOOKUP — sp_cpa_catalogo_lookup
// Autocomplete dentro de un tipo (5=bancos, 3=tipos persona moral, etc.).
// Filtra por tenant: ve globales + propios.
// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_catalogo_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid, lang: &str) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT soltarc.sp_cpa_catalogo_seed($1, $2)")
        .bind(tenant_id)
        .bind(lang)
        .fetch_one(pool)
        .await
}

// ─────────────────────────────────────────────
// LOOKUP — sp_cpa_catalogo_lookup
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, tipo: i16, q: &str, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM soltarc.sp_cpa_catalogo_lookup($1, $2, $3, $4)"
    )
    .bind(tipo)
    .bind(q)
    .bind(limit)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}
