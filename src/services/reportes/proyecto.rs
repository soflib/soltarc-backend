// Programa...: services::reportes::proyecto
// Descripción: Reportes de proyecto (partidas, árbol, auditoría, avance)
// Origen.....: oReportes.cs

use crate::dal::reportes as dal;
use crate::domain::models::reportes::{EstadoCuenta, PartidasArbol, PartidasPptoReporte, RegistroAvance};
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub async fn carga_partidas(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasPptoReporte>, ReturnCode> {
    dal::carga_partidas(pool, presupuesto).await
}

pub async fn arbol_tareas_proyecto(pool: &PgPool, proyecto: i32) -> Result<Vec<PartidasArbol>, ReturnCode> {
    dal::arbol_tareas_proyecto(pool, proyecto).await
}

pub async fn audita_xref(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasArbol>, ReturnCode> {
    dal::audita_xref(pool, presupuesto).await
}

pub async fn totales_ppto(pool: &PgPool, presupuesto: i32) -> Result<Decimal, ReturnCode> {
    dal::totales_ppto(pool, presupuesto).await
}

pub async fn ingresos(pool: &PgPool, proyecto: i32) -> Result<Vec<RegistroAvance>, ReturnCode> {
    dal::ingresos(pool, proyecto).await
}

pub async fn egresos(pool: &PgPool, proyecto: i32) -> Result<Vec<RegistroAvance>, ReturnCode> {
    dal::egresos(pool, proyecto).await
}

pub async fn estado_de_cuenta(pool: &PgPool, id: i32) -> Result<Vec<EstadoCuenta>, ReturnCode> {
    dal::estado_de_cuenta(pool, id).await
}
