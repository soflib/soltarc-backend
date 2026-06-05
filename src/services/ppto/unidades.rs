// Programa...: services::ppto::unidades
// Descripción: Capa de servicio para unidades de medida
// Origen.....: oUnidades.cs

use crate::dal::unidades as dal;
use crate::domain::models::unidades::Unidades;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, uni: &Unidades, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, uni, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambio(pool: &PgPool, uni: &Unidades, tenant_id: Uuid) -> ReturnCode {
    dal::cambio(pool, uni, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Unidades>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn obtiene_unidades(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Unidades>, ReturnCode> {
    dal::obtiene_unidades(pool, tenant_id).await
}

pub async fn carga_arbol(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Unidades>, ReturnCode> {
    dal::obtiene_unidades(pool, tenant_id).await
}
