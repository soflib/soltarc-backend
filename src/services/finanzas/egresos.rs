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

pub async fn alta(pool: &PgPool, egr: &Egresos) -> ReturnCode {
    dal::alta(pool, egr).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, egr: &Egresos) -> ReturnCode {
    dal::cambios(pool, egr).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Egresos>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn carga_egresos_proy_xref(pool: &PgPool, proyecto: i32) -> Result<Vec<Egresos>, ReturnCode> {
    dal::carga_egresos_proy_xref(pool, proyecto).await
}

pub async fn total_egresos(pool: &PgPool, proyecto: i32) -> Result<Decimal, ReturnCode> {
    dal::total_egresos(pool, proyecto).await
}

// ─────────────────────────────────────────────
// SEARCH — listado paginado con filtros (proyecto/proveedor/centro_costo/
//          fechas) + texto libre (ILIKE en referencia/comentario/proveedor/proyecto).
// ─────────────────────────────────────────────
pub async fn search(pool: &PgPool, filtros: &EgresosFilter) -> Result<PageOf<Egresos>, ReturnCode> {
    dal::search(pool, filtros).await
}
