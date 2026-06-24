// Programa...: centros_costo
// Descripción: Operaciones de la tabla cpa_CentrosCosto (multi-tenant)
// Origen.....: oCentrosCosto.cs
//
// Stored Procedures que usa:
//   sp_cpa_CentrosCostoAdd      → alta              (..., tenant_id)
//   sp_cpa_CentrosCostoDel      → baja              (id, tenant_id)
//   sp_cpa_CentrosCostoUpd      → cambios           (id, ..., tenant_id)
//   sp_cpa_CentrosCostoQry      → consulta por id   (id, tenant_id)
//   sp_cpa_CentrosCostosLstAll  → lista todos       (activos, tenant_id)
//   sp_cpa_centroscosto_lookup  → autocomplete      (q, limit, tenant_id)
//
// Todos los SPs filtran por (tenant_id IS NULL OR tenant_id = p_tenant_id) en
// lecturas y por igualdad estricta en del/upd (globales protegidos), igual que
// cpa_catalogos y cpa_proveedores.

use crate::domain::models::centros_costo::CentrosCosto;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_CentrosCostoAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cen: &CentrosCosto, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_CentrosCostoAdd($1, $2, $3, $4, $5)"
    )
    .bind(&cen.nombre)
    .bind(cen.tipo)
    .bind(&cen.comentarios)
    .bind(cen.activo)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_CentrosCostoDel
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_CentrosCostoDel($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_cpa_CentrosCostoUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cen: &CentrosCosto, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_CentrosCostoUpd($1, $2, $3, $4, $5, $6)"
    )
    .bind(cen.id)
    .bind(&cen.nombre)
    .bind(cen.tipo)
    .bind(&cen.comentarios)
    .bind(cen.activo)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_CentrosCostoQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<CentrosCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, CentrosCosto>(
        "SELECT * FROM arqeth.sp_cpa_CentrosCostoQry($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// OBTIENE TODO — sp_cpa_CentrosCostosLstAll
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<CentrosCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, CentrosCosto>(
        "SELECT * FROM arqeth.sp_cpa_CentrosCostosLstAll($1, $2)"
    )
    .bind(activos)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -51, afectado: 0, mensaje: "No hay centros de costo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_centroscosto_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid, lang: &str) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_cpa_centroscosto_seed($1, $2)")
        .bind(tenant_id)
        .bind(lang)
        .fetch_one(pool)
        .await
}

// ─────────────────────────────────────────────
// LOOKUP — sp_cpa_centroscosto_lookup
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM arqeth.sp_cpa_centroscosto_lookup($1, $2, $3)"
    )
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
