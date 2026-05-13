// Programa...: proveedores
// Descripción: Mantenimiento al catálogo de proveedores
// Origen.....: Proveedores.aspx.cs
//
// DAL que usa:
//   crate::dal::proveedores::alta
//   crate::dal::proveedores::baja
//   crate::dal::proveedores::cambio
//   crate::dal::proveedores::consulta
//   crate::dal::proveedores::carga_proveedores
//
// Catálogo auxiliar:
//   crate::dal::catalog_g::obtiene_por_tipo  (tipo 3 → Tipo Persona moral)
//   crate::dal::catalog_g::obtiene_por_tipo  (tipo 4 → Tipo proveedor / Giro)

use crate::dal::{catalog_g, proveedores as dal_prov};
use crate::domain::models::catalog_g::CatalogG;
use crate::domain::models::proveedores::Proveedores;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, prov: &Proveedores) -> ReturnCode {
    dal_prov::alta(pool, prov).await
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal_prov::baja(pool, id).await
}

// ─────────────────────────────────────────────
// CAMBIO
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, prov: &Proveedores) -> ReturnCode {
    dal_prov::cambio(pool, prov).await
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Proveedores>, ReturnCode> {
    dal_prov::consulta(pool, id).await
}

// ─────────────────────────────────────────────
// CARGA PROVEEDORES
// Reemplaza CargaProveedores(lProveedores, Activos)
// `activos: true` filtra sólo activos; false devuelve todos
// ─────────────────────────────────────────────
pub async fn carga_proveedores(pool: &PgPool, activos: bool) -> Result<Vec<Proveedores>, ReturnCode> {
    dal_prov::carga_proveedores(pool, activos).await
}

// ─────────────────────────────────────────────
// OBTIENE TIPOS (catálogo tipo 3 → Tipo Persona moral)
// Reemplaza oCat.ObtieneCats(3, ddTipo)
// ─────────────────────────────────────────────
pub async fn obtiene_tipos(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    catalog_g::obtiene_por_tipo(pool, 3).await
}

// ─────────────────────────────────────────────
// OBTIENE GIROS (catálogo tipo 4 → Tipo proveedor / Giro)
// Reemplaza oCat.ObtieneCats(4, ddGiro)
// ─────────────────────────────────────────────
pub async fn obtiene_giros(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    catalog_g::obtiene_por_tipo(pool, 4).await
}
