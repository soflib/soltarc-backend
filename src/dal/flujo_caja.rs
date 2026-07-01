// Programa...: flujo_caja
// Descripción: Flujo de Caja efectivo y bancos
// Origen.....: oFlujoCaja.cs
//
// Stored Procedures que usa:
//   sp_cpa_FlujoSaldos → consulta flujo de caja por rango de fechas

use crate::domain::models::flujo_caja::FlujoCaja;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

// ─────────────────────────────────────────────
// CONSULTA FLUJO — sp_cpa_FlujoSaldos
// ─────────────────────────────────────────────
pub async fn consulta_flujo(
    pool: &PgPool,
    fecha_saldo: Date,
    fecha_ini: Date,
    fecha_fin: Date,
) -> Result<Vec<FlujoCaja>, ReturnCode> {
    let result = sqlx::query_as::<_, FlujoCaja>(
        "SELECT * FROM soltarc.sp_cpa_FlujoSaldos($1, $2, $3)"
    )
    .bind(fecha_saldo)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -10, afectado: 0, mensaje: "No hay trx de flujo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}
