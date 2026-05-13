// Programa...: services::operaciones::proyectos
// Descripción: Lógica de negocio para proyectos
// Origen.....: oProyectos.cs
//
// DAL que usa:
//   crate::dal::proyectos::alta
//   crate::dal::proyectos::baja
//   crate::dal::proyectos::cambio
//   crate::dal::proyectos::gpo_usr_proyecto
//   crate::dal::proyectos::consulta
//   crate::dal::proyectos::llena_proyectos
//   crate::dal::proyectos::cliente_proyecto
//   crate::dal::proyectos::dir_proyecto
//   crate::dal::proyectos::total_ppto

use crate::dal::proyectos as dal;
use crate::domain::models::proyectos::Proyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, proy: &Proyectos) -> ReturnCode {
    dal::alta(pool, proy).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambio(pool: &PgPool, proy: &Proyectos) -> ReturnCode {
    dal::cambio(pool, proy).await
}

pub async fn gpo_usr_proyecto(pool: &PgPool, proyecto: i32, grupo: i32, usuario: i32) -> ReturnCode {
    dal::gpo_usr_proyecto(pool, proyecto, grupo, usuario).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Proyectos>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn llena_proyectos(pool: &PgPool, activos: bool) -> Result<Vec<Proyectos>, ReturnCode> {
    dal::llena_proyectos(pool, activos).await
}

pub async fn cliente_proyecto(pool: &PgPool, proyecto: i32) -> Result<String, ReturnCode> {
    dal::cliente_proyecto(pool, proyecto).await
}

pub async fn dir_proyecto(pool: &PgPool, proyecto: i32) -> Result<String, ReturnCode> {
    dal::dir_proyecto(pool, proyecto).await
}

pub async fn total_ppto(pool: &PgPool, proyecto: i32) -> Result<Decimal, ReturnCode> {
    dal::total_ppto(pool, proyecto).await
}

pub async fn lista_grupos(
    pool: &PgPool,
) -> Result<Vec<crate::domain::models::gn_grupos::GnGrupos>, ReturnCode> {
    crate::dal::gn_grupos::obtiene_todo(pool, true).await
}

pub async fn usuarios_grupo(
    pool: &PgPool,
    grupo_id: i32,
) -> Result<Vec<crate::domain::models::gn_usuarios::GnUsuarios>, ReturnCode> {
    let todos = crate::dal::gn_usuarios::obtiene_todo(pool).await?;
    Ok(todos.into_iter().filter(|u| u.grupo_negocio == grupo_id).collect())
}
