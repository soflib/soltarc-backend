// Programa...: services::clients::project_tasks
// Descripción: Tareas del proyecto con totales financieros para el portal de clientes
// Origen.....: Cte_ProyectosDetalleTareas.aspx.cs — CargaTareas, TotalEgresos

use crate::dal::{detalle_proyecto, egresos};
use crate::domain::models::detalle_proyectos::DetalleProyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub async fn carga_tareas(
    pool: &PgPool,
    proyecto: i32,
) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    detalle_proyecto::carga_tareas(pool, proyecto).await
}

pub async fn total_egresos(pool: &PgPool, proyecto: i32) -> Result<Decimal, ReturnCode> {
    egresos::total_egresos(pool, proyecto).await
}
