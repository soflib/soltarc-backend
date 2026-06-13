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
use uuid::Uuid;

pub async fn alta(pool: &PgPool, sdo: &SaldosBancos, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, sdo, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, sdo: &SaldosBancos, tenant_id: Uuid) -> ReturnCode {
    dal::cambios(pool, sdo, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<SaldosBancos>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn saldos_x_banco(pool: &PgPool, banco: i32, tenant_id: Uuid) -> Result<Vec<SaldosBancos>, ReturnCode> {
    dal::saldos_x_banco(pool, banco, tenant_id).await
}

pub async fn saldos_todos(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<SaldosBancos>, ReturnCode> {
    dal::saldos_todos(pool, tenant_id).await
}
