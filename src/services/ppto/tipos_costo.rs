// Programa...: services::ppto::tipos_costo
// Descripción: Capa de servicio para tipos de costo
// Origen.....: oTiposCosto.cs

use crate::dal::tipos_costo as dal;
use crate::domain::models::tipos_costo::TiposCosto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, tpo: &TiposCosto, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, tpo, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambio(pool: &PgPool, tpo: &TiposCosto, tenant_id: Uuid) -> ReturnCode {
    dal::cambio(pool, tpo, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<TiposCosto>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn carga_tipos(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<TiposCosto>, ReturnCode> {
    dal::carga_tipos(pool, activos, tenant_id).await
}
