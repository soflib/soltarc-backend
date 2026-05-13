// Programa...: services::clients::weekly_plan
// Descripción: Plan semanal Gantt del portal de clientes
// Origen.....: Cte_AvancePlanSemanal.aspx.cs — Fechas, CargaPartidas

use crate::dal::plan_semanal;
use crate::domain::models::plan_semanal::PartidasSemanal;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub struct Fechas {
    pub fecha_ini:   Date,
    pub fecha_fin:   Date,
    pub num_semanas: i32,
}

pub async fn fechas(pool: &PgPool, proyecto: i32) -> Result<Fechas, ReturnCode> {
    let f = plan_semanal::fechas(pool, proyecto).await?;
    Ok(Fechas {
        fecha_ini:   f.fecha_ini,
        fecha_fin:   f.fecha_fin,
        num_semanas: f.num_semanas,
    })
}

pub async fn carga_partidas(
    pool: &PgPool,
    proyecto: i32,
    fecha_ini: Date,
    nivel: i32,
) -> Result<Vec<PartidasSemanal>, ReturnCode> {
    plan_semanal::carga_partidas(pool, proyecto, fecha_ini, nivel).await
}
