// Programa...: services::sistema::gn_grupos
// Descripción: Capa de servicio para grupos de negocio
// Origen.....: oGNGrupos.cs

use crate::dal::gn_grupos as dal;
use crate::domain::models::gn_grupos::GnGrupos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, gpo: &GnGrupos) -> ReturnCode {
    dal::alta(pool, gpo).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, gpo: &GnGrupos) -> ReturnCode {
    dal::cambios(pool, gpo).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<GnGrupos>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn obtiene_todo(pool: &PgPool, cuales: bool) -> Result<Vec<GnGrupos>, ReturnCode> {
    dal::obtiene_todo(pool, cuales).await
}
