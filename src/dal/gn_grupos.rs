// Programa...: gn_grupos
// Descripción: Operaciones de la tabla GpoNegocio (multi-tenant)
// Origen.....: oGNGrupos.cs
//
// Stored Procedures que usa (todos con p_tenant_id):
//   sp_GN_GruposAdd    → alta              (..., tenant_id)
//   sp_GN_GruposDel    → baja              (id, tenant_id)
//   sp_GN_GruposUpd    → cambios           (id, ..., tenant_id)
//   sp_GN_GruposQry    → consulta por id   (id, tenant_id)
//   sp_GN_GruposLstAll → lista todos       (activos, tenant_id)
//   sp_gn_grupos_seed  → seed por tenant   (tenant_id)

use crate::domain::models::gn_grupos::GnGrupos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_GN_GruposAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, gpo: &GnGrupos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_GruposAdd($1, $2, $3, $4)"
    )
    .bind(&gpo.nombre)
    .bind(&gpo.descripcion)
    .bind(gpo.activo)
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
// BAJA — sp_GN_GruposDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet.
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_GN_GruposDel($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_GN_GruposUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, gpo: &GnGrupos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_GruposUpd($1, $2, $3, $4, $5)"
    )
    .bind(gpo.id)
    .bind(&gpo.nombre)
    .bind(&gpo.descripcion)
    .bind(gpo.activo)
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
// CONSULTA — sp_GN_GruposQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<GnGrupos>, ReturnCode> {
    let result = sqlx::query_as::<_, GnGrupos>(
        "SELECT * FROM arqeth.sp_GN_GruposQry($1, $2)"
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
// OBTIENE TODO — sp_GN_GruposLstAll
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool, cuales: bool, tenant_id: Uuid) -> Result<Vec<GnGrupos>, ReturnCode> {
    let result = sqlx::query_as::<_, GnGrupos>(
        "SELECT * FROM arqeth.sp_GN_GruposLstAll($1, $2)"
    )
    .bind(cuales)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -21, afectado: 0, mensaje: "No hay entradas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_gn_grupos_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid, lang: &str) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_gn_grupos_seed($1, $2)")
        .bind(tenant_id)
        .bind(lang)
        .fetch_one(pool)
        .await
}
