// Programa...: centros_costo
// Descripción: Operaciones de la tabla cpa_CentrosCosto
// Origen.....: oCentrosCosto.cs
//
// Stored Procedures que usa:
//   sp_cpa_CentrosCostoAdd      → alta
//   sp_cpa_CentrosCostoDel      → baja
//   sp_cpa_CentrosCostoUpd      → cambios
//   sp_cpa_CentrosCostoQry      → consulta por id
//   sp_cpa_CentrosCostosLstAll  → lista todos

use crate::domain::models::centros_costo::CentrosCosto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_CentrosCostoAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cen: &CentrosCosto) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_CentrosCostoAdd($1, $2, $3, $4)"
    )
    .bind(&cen.nombre)
    .bind(cen.tipo)
    .bind(&cen.comentarios)
    .bind(cen.activo)
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
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_CentrosCostoDel($1)"
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
// CAMBIOS — sp_cpa_CentrosCostoUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cen: &CentrosCosto) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_CentrosCostoUpd($1, $2, $3, $4, $5)"
    )
    .bind(cen.id)
    .bind(&cen.nombre)
    .bind(cen.tipo)
    .bind(&cen.comentarios)
    .bind(cen.activo)
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
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<CentrosCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, CentrosCosto>(
        "SELECT * FROM arqeth.sp_cpa_CentrosCostoQry($1)"
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
// OBTIENE TODO — sp_cpa_CentrosCostosLstAll
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool, activos: bool) -> Result<Vec<CentrosCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, CentrosCosto>(
        "SELECT * FROM arqeth.sp_cpa_CentrosCostosLstAll($1)"
    )
    .bind(activos)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -51, afectado: 0, mensaje: "No hay centros de costo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}
