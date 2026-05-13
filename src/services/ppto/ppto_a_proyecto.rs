// Programa...: services::ppto::ppto_a_proyecto
// Descripción: Capa de servicio para transferencia de presupuesto a proyecto
// Origen.....: oPPTOaProyecto.cs

use crate::dal::ppto_a_proyecto as dal;
use crate::domain::models::ppto_a_proyecto::NodoPartida;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub async fn consulta_numero_partidas(pool: &PgPool, proyecto: i32) -> ReturnCode {
    dal::consulta_numero_partidas(pool, proyecto).await
}

pub async fn carga_nodos(pool: &PgPool, ppto: i32, proyecto: i32) -> Result<Vec<NodoPartida>, ReturnCode> {
    dal::carga_nodos(pool, ppto, proyecto).await
}

pub async fn crea_partidas_proyecto(
    pool: &PgPool,
    ppto: i32,
    proyecto: i32,
    fecha_ini: Date,
    fecha_fin: Date,
    estado: i32,
    tipo: i32,
) -> ReturnCode {
    dal::crea_partidas_proyecto(pool, ppto, proyecto, fecha_ini, fecha_fin, estado, tipo).await
}

pub async fn obtiene_tipo_proyecto(pool: &PgPool, proyecto: i32) -> Result<i32, ReturnCode> {
    dal::obtiene_tipo_proyecto(pool, proyecto).await
}
