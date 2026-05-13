// Programa...: services::clients::dashboard
// Descripción: Dashboard del portal de clientes
// Origen.....: Cte_Inicial.aspx.cs — LlenaDetProyectos, TotalPPTO

use crate::dal::{finanzas, proyectos};
use crate::domain::models::finances::ResumenProyecto;
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub async fn llena_det_proyectos(
    pool: &PgPool,
    grupo: i32,
    usuario: i32,
    nivel: i32,
) -> Result<Vec<ResumenProyecto>, ReturnCode> {
    finanzas::llena_det_proyectos(pool, grupo, usuario, nivel).await
}

pub async fn total_ppto(pool: &PgPool, proyecto: i32) -> Result<Decimal, ReturnCode> {
    proyectos::total_ppto(pool, proyecto).await
}
