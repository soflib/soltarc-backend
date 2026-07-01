// Programa...: saldos_bancos
// Descripción: Operaciones de la tabla cpa_SaldosBancos
// Origen.....: oSaldosBancos.cs
//
// Stored Procedures que usa:
//   sp_cpa_SaldosBco_Add    → alta
//   sp_cpa_SaldosBco_DEL    → baja
//   sp_cpa_SaldosBco_Upd    → cambios
//   sp_cpa_SaldosBco_QRY    → consulta por id
//   sp_cpa_SaldosBco_LSTBco → saldos por banco
//   sp_cpa_SaldosBco_LSTALL → todos los saldos
//
// Nota: SaldosxBanco() y SaldosTodos() ligaban a GridView en C#.
//       Se migran a Vec<SaldosBancos>; la capa de presentación decide el binding.

use crate::domain::models::saldos_bancos::SaldosBancos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_SaldosBco_Add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, sdo: &SaldosBancos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_SaldosBco_Add($1, $2, $3, $4, $5)"
    )
    .bind(sdo.banco)
    .bind(sdo.ano)
    .bind(sdo.mes)
    .bind(sdo.monto)
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
// BAJA — sp_cpa_SaldosBco_DEL
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_SaldosBco_DEL($1, $2)"
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
// CAMBIOS — sp_cpa_SaldosBco_Upd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, sdo: &SaldosBancos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_SaldosBco_Upd($1, $2, $3, $4, $5, $6)"
    )
    .bind(sdo.id.unwrap_or(0))  // id es Option<i32> — 0 nunca debería llegar aquí
    .bind(sdo.banco)
    .bind(sdo.ano)
    .bind(sdo.mes)
    .bind(sdo.monto)
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
// CONSULTA — sp_cpa_SaldosBco_QRY
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<SaldosBancos>, ReturnCode> {
    let result = sqlx::query_as::<_, SaldosBancos>(
        "SELECT * FROM soltarc.sp_cpa_SaldosBco_QRY($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -41, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SALDOS POR BANCO — sp_cpa_SaldosBco_LSTBco
// ─────────────────────────────────────────────
pub async fn saldos_x_banco(pool: &PgPool, banco: i32, tenant_id: Uuid) -> Result<Vec<SaldosBancos>, ReturnCode> {
    let result = sqlx::query_as::<_, SaldosBancos>(
        "SELECT * FROM soltarc.sp_cpa_SaldosBco_LSTBco($1, $2)"
    )
    .bind(banco)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -51, afectado: 0, mensaje: "No hay entradas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SALDOS TODOS — sp_cpa_SaldosBco_LSTALL
// ─────────────────────────────────────────────
pub async fn saldos_todos(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<SaldosBancos>, ReturnCode> {
    let result = sqlx::query_as::<_, SaldosBancos>(
        "SELECT * FROM soltarc.sp_cpa_SaldosBco_LSTALL($1)"
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -61, afectado: 0, mensaje: "No hay saldos".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_saldosbco_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT soltarc.sp_cpa_saldosbco_seed($1)")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
}
