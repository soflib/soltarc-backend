// Programa...: gn_grupos
// Descripción: Operaciones de la tabla GpoNegocio
// Origen.....: oGNGrupos.cs
//
// Stored Procedures que usa:
//   sp_GN_GruposAdd    → alta
//   sp_GN_GruposDel    → baja
//   sp_GN_GruposUpd    → cambios
//   sp_GN_GruposQry    → consulta por id
//   sp_GN_GruposLstAll → lista todos (activos o no)

use crate::domain::models::gn_grupos::GnGrupos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_GN_GruposAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, gpo: &GnGrupos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_GruposAdd($1, $2, $3)"
    )
    .bind(&gpo.nombre)
    .bind(&gpo.descripcion)
    .bind(gpo.activo)
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
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_GruposDel($1)"
    )
    .bind(id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_GN_GruposUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, gpo: &GnGrupos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_GruposUpd($1, $2, $3, $4)"
    )
    .bind(gpo.id)
    .bind(&gpo.nombre)
    .bind(&gpo.descripcion)
    .bind(gpo.activo)
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
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<GnGrupos>, ReturnCode> {
    let result = sqlx::query_as::<_, GnGrupos>(
        "SELECT * FROM arqeth.sp_GN_GruposQry($1)"
    )
    .bind(id)
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
pub async fn obtiene_todo(pool: &PgPool, cuales: bool) -> Result<Vec<GnGrupos>, ReturnCode> {
    let result = sqlx::query_as::<_, GnGrupos>(
        "SELECT * FROM arqeth.sp_GN_GruposLstAll($1)"
    )
    .bind(cuales)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -21, afectado: 0, mensaje: "No hay entradas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}
