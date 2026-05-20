// Programa...: clientes
// Descripción: Mantenimiento al catálogo de clientes
// Origen.....: Clientes.aspx.cs
//
// DAL que usa:
//   crate::dal::clientes::alta
//   crate::dal::clientes::baja
//   crate::dal::clientes::cambios
//   crate::dal::clientes::consulta
//   crate::dal::clientes::obtiene_clientes
//
// Catálogo auxiliar:
//   crate::dal::catalog_g::obtiene_por_tipo  (tipo 3 → Tipo Persona moral)

use crate::dal::{catalog_g, clientes as dal_clientes};
use crate::domain::models::catalog_g::CatalogG;
use crate::domain::models::clients::Clientes;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cli: &Clientes) -> ReturnCode {
    dal_clientes::alta(pool, cli).await
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal_clientes::baja(pool, id).await
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cli: &Clientes) -> ReturnCode {
    dal_clientes::cambios(pool, cli).await
}

// ─────────────────────────────────────────────
// CONSULTA
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Clientes>, ReturnCode> {
    dal_clientes::consulta(pool, id).await
}

// ─────────────────────────────────────────────
// OBTIENE CLIENTES
// Reemplaza ObtieneClientes(lClientes, Activos)
// `activos: true` filtra sólo activos; false devuelve todos
// ─────────────────────────────────────────────
pub async fn obtiene_clientes(pool: &PgPool, activos: bool) -> Result<Vec<Clientes>, ReturnCode> {
    dal_clientes::obtiene_clientes(pool, activos).await
}

// ─────────────────────────────────────────────
// OBTIENE TIPOS DE CLIENTE (catálogo tipo 3 → Tipo Persona moral)
// Reemplaza oCatg.ObtieneCats(3, ddTipo)
// ─────────────────────────────────────────────
pub async fn obtiene_tipos(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    catalog_g::obtiene_por_tipo(pool, 3).await
}

// ─────────────────────────────────────────────
// LOOKUP — autocomplete clientes activos
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, limit: i32) -> Result<Vec<LookupItem>, ReturnCode> {
    dal_clientes::lookup(pool, q, limit).await
}
