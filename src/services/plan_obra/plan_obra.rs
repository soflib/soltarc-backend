// Programa...: services::plan_obra::plan_obra
// Origen.....: oPlanObra.cs

use crate::dal::plan_obra::{self as dal, PlanStatus};
use crate::domain::models::plan_obra::PlanObra;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub async fn partida_upd_fecha(pool: &PgPool, pla: &PlanObra) -> ReturnCode {
    dal::partida_upd_fecha(pool, pla).await
}

pub async fn partida_proyecto(pool: &PgPool, proyecto: i32) -> Result<Vec<PlanObra>, ReturnCode> {
    dal::partida_proyecto(pool, proyecto).await
}

pub async fn obtiene_avance(pool: &PgPool, proyecto: i32, nivel: i32) -> Result<Vec<PlanObra>, ReturnCode> {
    dal::obtiene_avance(pool, proyecto, nivel).await
}

pub async fn existe_plan(pool: &PgPool, proyecto: i32) -> Result<PlanStatus, ReturnCode> {
    dal::existe_plan(pool, proyecto).await
}

pub async fn crea_plan(
    pool: &PgPool,
    proyecto: i32,
    fecha_ini: Date,
    fecha_fin: Date,
    estado: i32,
) -> ReturnCode {
    dal::crea_plan(pool, proyecto, fecha_ini, fecha_fin, estado).await
}

pub async fn descendientes_nodo(pool: &PgPool, nodo: &str) -> Result<Vec<PlanObra>, ReturnCode> {
    dal::descendientes_nodo(pool, nodo).await
}
