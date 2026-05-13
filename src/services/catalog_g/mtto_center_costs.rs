// Programa...: mtto_centros_costo
// Descripción: Mantenimiento a Centros de costo
// Origen.....: MttoCentrosCosto.aspx.cs
//
// DAL que usa:
//   crate::dal::centros_costo::alta
//   crate::dal::centros_costo::baja
//   crate::dal::centros_costo::cambios
//   crate::dal::centros_costo::consulta
//   crate::dal::centros_costo::obtiene_todo

use crate::dal::centros_costo;
use crate::domain::models::centros_costo::CentrosCosto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cen: &CentrosCosto) -> ReturnCode {
    centros_costo::alta(pool, cen).await
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    centros_costo::baja(pool, id).await
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cen: &CentrosCosto) -> ReturnCode {
    centros_costo::cambios(pool, cen).await
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<CentrosCosto>, ReturnCode> {
    centros_costo::consulta(pool, id).await
}

// ─────────────────────────────────────────────
// OBTIENE CENTROS
// Reemplaza ObtieneCentros(lbCentros) y su variante con cbActivos
// `activos: true`  devuelve sólo activos  (comportamiento por defecto en C#)
// `activos: false` devuelve todos
// ─────────────────────────────────────────────
pub async fn obtiene_centros(pool: &PgPool, activos: bool) -> Result<Vec<CentrosCosto>, ReturnCode> {
    centros_costo::obtiene_todo(pool, activos).await
}
