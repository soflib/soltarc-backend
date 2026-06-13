// Programa...: services::sistema::gn_usuarios
// Descripción: Capa de servicio para usuarios de grupos de negocio (multi-tenant)
// Origen.....: oGNUsuarios.cs

use crate::dal::gn_usuarios as dal;
use crate::domain::models::gn_usuarios::GnUsuarios;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, usr: &GnUsuarios, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, usr, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, usr: &GnUsuarios, tenant_id: Uuid) -> ReturnCode {
    dal::cambios(pool, usr, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<GnUsuarios>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn obtiene_todo(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<GnUsuarios>, ReturnCode> {
    dal::obtiene_todo(pool, tenant_id).await
}
