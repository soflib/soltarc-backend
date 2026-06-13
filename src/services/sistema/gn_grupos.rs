// Programa...: services::sistema::gn_grupos
// Descripción: Capa de servicio para grupos de negocio (multi-tenant)
// Origen.....: oGNGrupos.cs

use crate::dal::gn_grupos as dal;
use crate::domain::models::gn_grupos::GnGrupos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, gpo: &GnGrupos, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, gpo, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, gpo: &GnGrupos, tenant_id: Uuid) -> ReturnCode {
    dal::cambios(pool, gpo, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<GnGrupos>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn obtiene_todo(pool: &PgPool, cuales: bool, tenant_id: Uuid) -> Result<Vec<GnGrupos>, ReturnCode> {
    dal::obtiene_todo(pool, cuales, tenant_id).await
}
