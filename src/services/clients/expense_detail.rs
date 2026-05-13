// Programa...: services::clients::expense_detail
// Descripción: Detalle de gastos por partida para el portal de clientes
// Origen.....: Cte_DetalleProyXrefGasto.aspx.cs — ConsultaPartidasXref

use crate::dal::detalle_proyecto;
use crate::domain::models::detalle_proyectos::DetalleProyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn consulta_partidas_xref(
    pool: &PgPool,
    proyecto: i32,
) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    detalle_proyecto::consulta_partidas_xref(pool, proyecto).await
}
