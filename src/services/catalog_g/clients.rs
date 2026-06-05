// Programa...: clientes
// Descripción: Mantenimiento al catálogo de clientes (multi-tenant)
// Origen.....: Clientes.aspx.cs

use crate::dal::{catalog_g, clientes as dal_clientes};
use crate::domain::models::catalog_g::CatalogG;
use crate::domain::models::clients::Clientes;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, cli: &Clientes, tenant_id: Uuid) -> ReturnCode {
    dal_clientes::alta(pool, cli, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal_clientes::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, cli: &Clientes, tenant_id: Uuid) -> ReturnCode {
    dal_clientes::cambios(pool, cli, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Clientes>, ReturnCode> {
    dal_clientes::consulta(pool, id, tenant_id).await
}

pub async fn obtiene_clientes(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<Clientes>, ReturnCode> {
    dal_clientes::obtiene_clientes(pool, activos, tenant_id).await
}

// OBTIENE TIPOS DE CLIENTE (catálogo tipo 3 → Tipo Persona moral)
pub async fn obtiene_tipos(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    catalog_g::obtiene_por_tipo(pool, 3, tenant_id).await
}

pub async fn lookup(pool: &PgPool, q: &str, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    dal_clientes::lookup(pool, q, limit, tenant_id).await
}
