// Programa...: services::sistema::gn_usuarios
// Descripción: Capa de servicio para usuarios de grupos de negocio
// Origen.....: oGNUsuarios.cs

use crate::dal::gn_usuarios as dal;
use crate::domain::models::gn_usuarios::GnUsuarios;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, usr: &GnUsuarios) -> ReturnCode {
    dal::alta(pool, usr).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, usr: &GnUsuarios) -> ReturnCode {
    dal::cambios(pool, usr).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<GnUsuarios>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn obtiene_todo(pool: &PgPool) -> Result<Vec<GnUsuarios>, ReturnCode> {
    dal::obtiene_todo(pool).await
}
