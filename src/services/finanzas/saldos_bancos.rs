// Programa...: services::finanzas::saldos_bancos
// Descripción: Capa de servicio para saldos de bancos
// Origen.....: oSaldosBancos.cs
//
// DAL que usa:
//   crate::dal::saldos_bancos::{alta, baja, cambios, consulta,
//                                saldos_x_banco, saldos_todos}

use crate::dal::saldos_bancos as dal;
use crate::domain::models::saldos_bancos::SaldosBancos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, sdo: &SaldosBancos) -> ReturnCode {
    dal::alta(pool, sdo).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, sdo: &SaldosBancos) -> ReturnCode {
    dal::cambios(pool, sdo).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<SaldosBancos>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn saldos_x_banco(pool: &PgPool, banco: i32) -> Result<Vec<SaldosBancos>, ReturnCode> {
    dal::saldos_x_banco(pool, banco).await
}

pub async fn saldos_todos(pool: &PgPool) -> Result<Vec<SaldosBancos>, ReturnCode> {
    dal::saldos_todos(pool).await
}
