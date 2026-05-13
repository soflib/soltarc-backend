// Programa...: services::finanzas::ingresos
// Descripción: Capa de servicio para ingresos
// Origen.....: oIngresos.cs
//
// DAL que usa:
//   crate::dal::ingresos::{alta, baja, cambios, consulta}

use crate::dal::ingresos as dal;
use crate::domain::models::ingresos::Ingresos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, ing: &Ingresos) -> ReturnCode {
    dal::alta(pool, ing).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, ing: &Ingresos) -> ReturnCode {
    dal::cambios(pool, ing).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Ingresos>, ReturnCode> {
    dal::consulta(pool, id).await
}
