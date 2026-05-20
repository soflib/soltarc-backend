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

pub async fn alta(pool: &PgPool, ing: &Ingresos) -> ReturnCode {
    dal::alta(pool, ing).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, ing: &Ingresos) -> ReturnCode {
    dal::cambios(pool, ing).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Ingresos>, ReturnCode> {
    dal::consulta(pool, id).await
}

// ─────────────────────────────────────────────
// LISTA — sin filtro de texto, opcionalmente por proyecto/cliente
// ─────────────────────────────────────────────
pub async fn lista(pool: &PgPool, proyecto: Option<i32>, cliente: Option<i32>) -> Result<Vec<Ingresos>, ReturnCode> {
    dal::lista(pool, proyecto, cliente).await
}

// ─────────────────────────────────────────────
// SEARCH — listado paginado con filtros + texto libre (ILIKE)
// ─────────────────────────────────────────────
pub async fn search(pool: &PgPool, filtros: &IngresosFilter) -> Result<PageOf<Ingresos>, ReturnCode> {
    dal::search(pool, filtros).await
}
