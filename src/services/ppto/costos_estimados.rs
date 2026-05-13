// Programa...: services::ppto::costos_estimados
// Descripción: Capa de servicio para costos estimados
// Origen.....: oCostosEstimados.cs

use crate::dal::costos_estimados as dal;
use crate::domain::models::costos_estimados::CostosEstimados;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, cos: &CostosEstimados) -> ReturnCode {
    dal::alta(pool, cos).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, cos: &CostosEstimados) -> ReturnCode {
    dal::cambios(pool, cos).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<CostosEstimados>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn carga_arbol(pool: &PgPool, activos: bool) -> Result<Vec<CostosEstimados>, ReturnCode> {
    dal::obtiene_activos(pool, activos).await
}
