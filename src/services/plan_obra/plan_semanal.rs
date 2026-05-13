// Programa...: services::plan_obra::plan_semanal
// Origen.....: oPlanSemanal.cs

use crate::dal::plan_semanal as dal;
use crate::dal::plan_semanal::Fechas;
use crate::domain::models::plan_semanal::PartidasSemanal;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub async fn fechas(pool: &PgPool, proyecto: i32) -> Result<Fechas, ReturnCode> {
    dal::fechas(pool, proyecto).await
}

pub async fn carga_partidas(
    pool: &PgPool,
    proyecto: i32,
    fecha_ini: Date,
    nivel: i32,
) -> Result<Vec<PartidasSemanal>, ReturnCode> {
    dal::carga_partidas(pool, proyecto, fecha_ini, nivel).await
}
