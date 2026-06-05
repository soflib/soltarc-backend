// Programa...: services::ppto::costos_estimados
// Descripción: Capa de servicio para costos estimados
// Origen.....: oCostosEstimados.cs

use crate::dal::costos_estimados as dal;
use crate::domain::models::costos_estimados::CostosEstimados;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, cos: &CostosEstimados, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, cos, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambios(pool: &PgPool, cos: &CostosEstimados, tenant_id: Uuid) -> ReturnCode {
    dal::cambios(pool, cos, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<CostosEstimados>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

pub async fn carga_arbol(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<CostosEstimados>, ReturnCode> {
    dal::obtiene_activos(pool, activos, tenant_id).await
}
