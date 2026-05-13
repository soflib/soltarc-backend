// Programa...: services::sistema::seguridad
// Descripción: Capa de servicio para variables de seguridad
// Origen.....: oSeguridad.cs

use crate::dal::seguridad as dal;
use crate::domain::models::seguridad::Seguridad;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn carga_variables(
    pool: &PgPool,
    gpo_id: i32,
    usr_id: i32,
    usr_nivel: i32,
    usr_activo: bool,
) -> Result<Seguridad, ReturnCode> {
    dal::carga_variables(pool, gpo_id, usr_id, usr_nivel, usr_activo).await
}
