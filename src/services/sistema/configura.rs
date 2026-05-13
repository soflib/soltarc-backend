// Programa...: services::sistema::configura
// Descripción: Capa de servicio para configuración del sistema
// Origen.....: oConfigura.cs

use crate::dal::configura as dal;
use crate::domain::models::configura::Configura;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn cambia_configuracion(pool: &PgPool, cfg: &Configura) -> ReturnCode {
    dal::cambia_configuracion(pool, cfg).await
}

pub async fn carga_configuracion(pool: &PgPool) -> Result<Option<Configura>, ReturnCode> {
    dal::carga_configuracion(pool).await
}
