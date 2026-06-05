// Programa...: costos_estimados
// Descripción: Operaciones de la tabla cpa_CostosEstimados
// Origen.....: oCostosEstimados.cs
//
// Stored Procedures que usa:
//   ppto_sp_CostosEstimados_Add    → alta
//   ppto_sp_CostosEstimados_DEL    → baja
//   ppto_sp_CostosEstimados_UPD    → cambios
//   ppto_sp_CostosEstimados_QRY    → consulta por id
//   ppto_sp_CostosEstimados_LSTACT → lista activos / todos
//
// Nota: CargaArbol() construía HTML server-side en C# (WebForms).
//       En Rust ese concern pertenece al frontend; este módulo
//       expone obtiene_activos() para que la capa de presentación
//       arme la estructura que necesite.

use crate::domain::models::costos_estimados::CostosEstimados;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — ppto_sp_CostosEstimados_Add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cos: &CostosEstimados, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_CostosEstimados_Add($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(cos.tipo)
    .bind(&cos.nombre)
    .bind(&cos.descripcion)
    .bind(cos.unidad)
    .bind(cos.fecha)
    .bind(cos.importe)
    .bind(cos.activo)
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
// BAJA — ppto_sp_CostosEstimados_DEL
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_CostosEstimados_DEL($1, $2)"
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
// CAMBIOS — ppto_sp_CostosEstimados_UPD
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cos: &CostosEstimados, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_CostosEstimados_UPD($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(cos.id)
    .bind(cos.tipo)
    .bind(&cos.nombre)
    .bind(&cos.descripcion)
    .bind(cos.unidad)
    .bind(cos.fecha)
    .bind(cos.importe)
    .bind(cos.activo)
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
// CONSULTA — ppto_sp_CostosEstimados_QRY
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<CostosEstimados>, ReturnCode> {
    let result = sqlx::query_as::<_, CostosEstimados>(
        "SELECT * FROM arqeth.ppto_sp_CostosEstimados_QRY($1, $2)"
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
// OBTIENE ACTIVOS — ppto_sp_CostosEstimados_LSTACT
// Reemplaza CargaArbol(): devuelve los datos crudos para que
// la capa de presentación construya la jerarquía que necesite.
// ─────────────────────────────────────────────
pub async fn obtiene_activos(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<CostosEstimados>, ReturnCode> {
    let result = sqlx::query_as::<_, CostosEstimados>(
        "SELECT * FROM arqeth.ppto_sp_CostosEstimados_LSTACT($1, $2)"
    )
    .bind(activos)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -71, afectado: 0, mensaje: "No hay Costos estimados".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -75, afectado: 0, mensaje: e.to_string() }),
    }
}
