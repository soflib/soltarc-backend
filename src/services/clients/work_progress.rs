// Programa...: services::clients::work_progress
// Descripción: Avance de obra del portal de clientes
// Origen.....: Cte_AvanceDeObra.aspx.cs — Ingresos, Egresos, Consultas, NombreCliente

use crate::dal::{clientes, proyectos, reportes};
use crate::domain::models::proyectos::Proyectos;
use crate::domain::models::reportes::RegistroAvance;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn consulta_proyecto(pool: &PgPool, proyecto: i32) -> Result<Option<Proyectos>, ReturnCode> {
    proyectos::consulta(pool, proyecto).await
}

pub async fn ingresos(pool: &PgPool, proyecto: i32) -> Result<Vec<RegistroAvance>, ReturnCode> {
    reportes::ingresos(pool, proyecto).await
}

pub async fn egresos(pool: &PgPool, proyecto: i32) -> Result<Vec<RegistroAvance>, ReturnCode> {
    reportes::egresos(pool, proyecto).await
}

pub async fn nombre_cliente(pool: &PgPool, cliente: i32) -> ReturnCode {
    clientes::nombre_cliente(pool, cliente).await
}
