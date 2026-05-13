// Programa...: services::ppto::tipos_costo
// Descripción: Capa de servicio para tipos de costo
// Origen.....: oTiposCosto.cs

use crate::dal::tipos_costo as dal;
use crate::domain::models::tipos_costo::TiposCosto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, tpo: &TiposCosto) -> ReturnCode {
    dal::alta(pool, tpo).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambio(pool: &PgPool, tpo: &TiposCosto) -> ReturnCode {
    dal::cambio(pool, tpo).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<TiposCosto>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn carga_tipos(pool: &PgPool, activos: bool) -> Result<Vec<TiposCosto>, ReturnCode> {
    dal::carga_tipos(pool, activos).await
}
