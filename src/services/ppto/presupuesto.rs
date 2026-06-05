// Programa...: services::ppto::presupuesto
// Descripción: Capa de servicio para presupuestos
// Origen.....: oPresupuesto.cs

use crate::dal::presupuesto as dal;
use crate::domain::models::lookup::LookupItem;
use crate::domain::models::presupuesto::Presupuesto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, ppto: &Presupuesto, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, ppto, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambio(pool: &PgPool, ppto: &Presupuesto, tenant_id: Uuid) -> ReturnCode {
    dal::cambio(pool, ppto, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Presupuesto>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn carga_pptos(
    pool: &PgPool,
    gpo_neg: i32,
    gpo_user_id: i32,
    usr_nivel: i32,
    activos: bool,
    tenant_id: Uuid,
) -> Result<Vec<Presupuesto>, ReturnCode> {
    dal::carga_presupuestos(pool, gpo_neg, gpo_user_id, usr_nivel, activos, tenant_id).await
}

// ─────────────────────────────────────────────
// LOOKUP — autocomplete presupuestos activos, filtrable por cliente
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, cliente: Option<i32>, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    dal::lookup(pool, q, cliente, limit, tenant_id).await
}
