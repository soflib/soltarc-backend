// Programa...: services::clients::project_tasks
// Descripción: Tareas del proyecto con totales financieros para el portal de clientes
// Origen.....: Cte_ProyectosDetalleTareas.aspx.cs — CargaTareas, TotalEgresos

use crate::dal::{detalle_proyecto, egresos};
use crate::domain::models::detalle_proyectos::DetalleProyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn carga_tareas(
    pool: &PgPool,
    proyecto: i32,
) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    detalle_proyecto::carga_tareas(pool, proyecto).await
}

pub async fn total_egresos(pool: &PgPool, proyecto: i32, tenant_id: Uuid) -> Result<Decimal, ReturnCode> {
    egresos::total_egresos(pool, proyecto, tenant_id).await
}

// Mapa id→nombre de los estados de partida (tipo=8) del tenant, para resolver el
// `estado` numérico de cada tarea a su nombre legible. Ver super::catalogo_map.
pub async fn estados_partida_map(pool: &PgPool, tenant_id: Uuid) -> HashMap<i32, String> {
    super::catalogo_map(pool, super::CAT_ESTADO_PARTIDAS, tenant_id).await
}
