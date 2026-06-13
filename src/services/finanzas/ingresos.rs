// Programa...: services::finanzas::ingresos
// Descripción: Capa de servicio para ingresos
// Origen.....: oIngresos.cs
//
// DAL que usa:
//   crate::dal::ingresos::{alta, baja, cambios, consulta}

use crate::dal::ingresos as dal;
use crate::domain::models::ingresos::{Ingresos, IngresosFilter};
use crate::domain::models::lookup::PageOf;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, ing: &Ingresos, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, ing, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, ing: &Ingresos, tenant_id: Uuid) -> ReturnCode {
    dal::cambios(pool, ing, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Ingresos>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

// ─────────────────────────────────────────────
// LISTA — sin filtro de texto, opcionalmente por proyecto/cliente
// ─────────────────────────────────────────────
pub async fn lista(pool: &PgPool, proyecto: Option<i32>, cliente: Option<i32>, tenant_id: Uuid) -> Result<Vec<Ingresos>, ReturnCode> {
    dal::lista(pool, proyecto, cliente, tenant_id).await
}

// ─────────────────────────────────────────────
// SEARCH — listado paginado con filtros + texto libre (ILIKE)
// ─────────────────────────────────────────────
pub async fn search(pool: &PgPool, filtros: &IngresosFilter, tenant_id: Uuid) -> Result<PageOf<Ingresos>, ReturnCode> {
    dal::search(pool, filtros, tenant_id).await
}
