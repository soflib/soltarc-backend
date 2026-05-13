// Programa...: services::clients::account_statement
// Descripción: Estado de cuenta del portal de clientes
// Origen.....: Cte_Estado_De_Cuenta.aspx.cs — EstadoDeCuenta, NombreCliente

use crate::dal::{clientes, reportes};
use crate::domain::models::reportes::EstadoCuenta;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn estado_de_cuenta(
    pool: &PgPool,
    cliente: i32,
) -> Result<Vec<EstadoCuenta>, ReturnCode> {
    reportes::estado_de_cuenta(pool, cliente).await
}

pub async fn nombre_cliente(pool: &PgPool, cliente: i32) -> ReturnCode {
    clientes::nombre_cliente(pool, cliente).await
}
