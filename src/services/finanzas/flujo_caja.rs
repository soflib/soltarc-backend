// Programa...: services::finanzas::flujo_caja
// Origen.....: oFlujoCaja.cs

use crate::dal::flujo_caja;
use crate::domain::models::flujo_caja::FlujoCaja;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub async fn consulta_flujo(
    pool: &PgPool,
    fecha_saldo: Date,
    fecha_ini: Date,
    fecha_fin: Date,
) -> Result<Vec<FlujoCaja>, ReturnCode> {
    flujo_caja::consulta_flujo(pool, fecha_saldo, fecha_ini, fecha_fin).await
}
