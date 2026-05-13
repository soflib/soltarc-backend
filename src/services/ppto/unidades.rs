// Programa...: services::ppto::unidades
// Descripción: Capa de servicio para unidades de medida
// Origen.....: oUnidades.cs

use crate::dal::unidades as dal;
use crate::domain::models::unidades::Unidades;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, uni: &Unidades) -> ReturnCode {
    dal::alta(pool, uni).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambio(pool: &PgPool, uni: &Unidades) -> ReturnCode {
    dal::cambio(pool, uni).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Unidades>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn obtiene_unidades(pool: &PgPool) -> Result<Vec<Unidades>, ReturnCode> {
    dal::obtiene_unidades(pool).await
}

pub async fn carga_arbol(pool: &PgPool) -> Result<Vec<Unidades>, ReturnCode> {
    dal::obtiene_unidades(pool).await
}
