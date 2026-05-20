// Programa...: catalog_g
// Descripción: Operaciones de la tabla cpa_Catalogos
// Origen.....: oCatalogoG.cs
//
// Stored Procedures que usa:
//   sp_cpa_catalogo_add       → alta
//   sp_cpa_catalogo_del       → baja
//   sp_cpa_catalogo_upd       → cambios
//   sp_cpa_catalogo_qry       → consulta por id
//   sp_cpa_catalogo_lst_all   → lista todos
//   sp_cpa_catalogo_qry_tipo  → lista por tipo
//   sp_cpa_catalogo_lst_tipos → lista tipos distintos

use crate::domain::models::catalog_g::CatalogG;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use tracing::{debug, error, info};

// ─────────────────────────────────────────────
// ALTA — sp_cpa_catalogo_add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cat: &CatalogG) -> ReturnCode {
    debug!("dal::catalog_g::alta → tipo={:?} nombre={:?} activo={:?}", cat.tipo, cat.nombre, cat.activo);

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_catalogo_add($1, $2, $3, $4)"
    )
    .bind(cat.tipo)
    .bind(&cat.nombre)
    .bind(cat.activo)
    .bind(&cat.comentarios)
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
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    debug!("dal::catalog_g::baja → id={}", id);

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_catalogo_del($1)"
    )
    .bind(id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => {
            info!("dal::catalog_g::baja ← OK afectado={}", n);
            ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() }
        }
        Ok(n) => {
            error!("dal::catalog_g::baja ← cancelada, sp retornó {}", n);
            ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() }
        }
        Err(e) => {
            error!("dal::catalog_g::baja ← DB error: {}", e);
            ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }
        }
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_cpa_catalogo_upd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cat: &CatalogG) -> ReturnCode {
    let id = cat.id.unwrap_or(0);
    debug!("dal::catalog_g::cambios → id={} tipo={:?} nombre={:?}", id, cat.tipo, cat.nombre);

    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_catalogo_upd($1, $2, $3, $4, $5)"
    )
    .bind(id)
    .bind(cat.tipo)
    .bind(&cat.nombre)
    .bind(cat.activo)
    .bind(&cat.comentarios)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => {
            info!("dal::catalog_g::cambios ← OK afectado={}", n);
            ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() }
        }
        Ok(n) => {
            error!("dal::catalog_g::cambios ← cancelada, sp retornó {}", n);
            ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() }
        }
        Err(e) => {
            error!("dal::catalog_g::cambios ← DB error: {}", e);
            ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() }
        }
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_catalogo_qry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::consulta → id={}", id);

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM arqeth.sp_cpa_catalogo_qry($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(r)) => {
            info!("dal::catalog_g::consulta ← encontrado nombre={:?}", r.nombre);
            Ok(Some(r))
        }
        Ok(None) => {
            info!("dal::catalog_g::consulta ← no existe id={}", id);
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
pub async fn obtiene_todo(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::obtiene_todo →");

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM arqeth.sp_cpa_catalogo_lst_all()"
    )
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
pub async fn obtiene_por_tipo(pool: &PgPool, tipo: i32) -> Result<Vec<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::obtiene_por_tipo → tipo={}", tipo);

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM arqeth.sp_cpa_catalogo_qry_tipo($1)"
    )
    .bind(tipo as i16)
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
pub async fn obtiene_tipos(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    debug!("dal::catalog_g::obtiene_tipos →");

    let result = sqlx::query_as::<_, CatalogG>(
        "SELECT * FROM arqeth.sp_cpa_catalogo_lst_tipos()"
    )
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
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, tipo: i16, q: &str, limit: i32) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM arqeth.sp_cpa_catalogo_lookup($1, $2, $3)"
    )
    .bind(tipo)
    .bind(q)
    .bind(limit)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}
