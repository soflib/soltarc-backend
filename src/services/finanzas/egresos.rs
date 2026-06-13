// Programa...: services::finanzas::egresos
// Descripción: Capa de servicio para egresos
// Origen.....: oEgresos.cs
//
// DAL que usa:
//   crate::dal::egresos::{alta, baja, cambios, consulta,
//                          carga_egresos_proy_xref, total_egresos}

use crate::dal::egresos as dal;
use crate::domain::models::egresos::{Egresos, EgresosFilter};
use crate::domain::models::lookup::PageOf;
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, egr: &Egresos, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, egr, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, egr: &Egresos, tenant_id: Uuid) -> ReturnCode {
    dal::cambios(pool, egr, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Egresos>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn carga_egresos_proy_xref(pool: &PgPool, proyecto: i32, tenant_id: Uuid) -> Result<Vec<Egresos>, ReturnCode> {
    dal::carga_egresos_proy_xref(pool, proyecto, tenant_id).await
}

pub async fn total_egresos(pool: &PgPool, proyecto: i32, tenant_id: Uuid) -> Result<Decimal, ReturnCode> {
    dal::total_egresos(pool, proyecto, tenant_id).await
}

// ─────────────────────────────────────────────
// SEARCH — listado paginado con filtros (proyecto/proveedor/centro_costo/
//          fechas) + texto libre (ILIKE en referencia/comentario/proveedor/proyecto).
// ─────────────────────────────────────────────
pub async fn search(pool: &PgPool, filtros: &EgresosFilter, tenant_id: Uuid) -> Result<PageOf<Egresos>, ReturnCode> {
    dal::search(pool, filtros, tenant_id).await
}
