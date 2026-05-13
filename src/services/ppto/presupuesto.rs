// Programa...: services::ppto::presupuesto
// Descripción: Capa de servicio para presupuestos
// Origen.....: oPresupuesto.cs

use crate::dal::presupuesto as dal;
use crate::domain::models::presupuesto::Presupuesto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, ppto: &Presupuesto) -> ReturnCode {
    dal::alta(pool, ppto).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambio(pool: &PgPool, ppto: &Presupuesto) -> ReturnCode {
    dal::cambio(pool, ppto).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Presupuesto>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn carga_pptos(
    pool: &PgPool,
    gpo_neg: i32,
    gpo_user_id: i32,
    usr_nivel: i32,
    activos: bool,
) -> Result<Vec<Presupuesto>, ReturnCode> {
    dal::carga_presupuestos(pool, gpo_neg, gpo_user_id, usr_nivel, activos).await
}
