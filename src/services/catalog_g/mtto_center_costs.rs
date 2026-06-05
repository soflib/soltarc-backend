// Programa...: mtto_centros_costo
// Descripción: Mantenimiento a Centros de costo (multi-tenant)
// Origen.....: MttoCentrosCosto.aspx.cs

use crate::dal::centros_costo;
use crate::domain::models::centros_costo::CentrosCosto;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, cen: &CentrosCosto, tenant_id: Uuid) -> ReturnCode {
    centros_costo::alta(pool, cen, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    centros_costo::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, cen: &CentrosCosto, tenant_id: Uuid) -> ReturnCode {
    centros_costo::cambios(pool, cen, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<CentrosCosto>, ReturnCode> {
    centros_costo::consulta(pool, id, tenant_id).await
}

pub async fn obtiene_centros(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<CentrosCosto>, ReturnCode> {
    centros_costo::obtiene_todo(pool, activos, tenant_id).await
}

pub async fn lookup(pool: &PgPool, q: &str, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    centros_costo::lookup(pool, q, limit, tenant_id).await
}
