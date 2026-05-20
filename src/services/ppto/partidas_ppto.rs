// Programa...: services::ppto::partidas_ppto
// Descripción: Capa de servicio para partidas de presupuesto
// Origen.....: oPartidasPPTO.cs

use crate::dal::partidas_ppto as dal;
use crate::domain::models::partidas_ppto::{PartidaBuscada, PartidasPpto};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, par: &PartidasPpto) -> ReturnCode {
    dal::alta(pool, par).await
}

pub async fn borra(pool: &PgPool, id: i32, nodo: &str) -> ReturnCode {
    dal::borra(pool, id, nodo).await
}

pub async fn cambio(pool: &PgPool, par: &PartidasPpto) -> ReturnCode {
    dal::cambio(pool, par).await
}

pub async fn carga_partidas(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasPpto>, ReturnCode> {
    dal::carga_partidas(pool, presupuesto).await
}

pub async fn partidas_actualiza_nodo(pool: &PgPool, id: i32, ppto: i32, nuevo_nodo: &str) -> ReturnCode {
    dal::partidas_actualiza_nodo(pool, id, ppto, nuevo_nodo).await
}

pub async fn nuevo_nodo_adiciona(pool: &PgPool, ppto: i32, nodo: &str, nivel: i32) -> Result<String, ReturnCode> {
    dal::nuevo_nodo_adiciona(pool, ppto, nodo, nivel).await
}

pub async fn carga_2_nivel(pool: &PgPool, nodo: i32, ppto: i32) -> Result<Vec<PartidasPpto>, ReturnCode> {
    dal::carga_2_nivel(pool, nodo, ppto).await
}

// ─────────────────────────────────────────────
// ARBOL — alias semántico de carga_partidas (devuelve el árbol completo con nivel).
// Se expone con nombre `arbol` para reflejar el endpoint público /ppto/partidas/arbol.
// ─────────────────────────────────────────────
pub async fn arbol(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasPpto>, ReturnCode> {
    dal::carga_partidas(pool, presupuesto).await
}

// ─────────────────────────────────────────────
// BUSCAR — búsqueda por concepto dentro de un presupuesto, incluyendo
// `ruta` ancestral ("Preliminares > Excavación").
// ─────────────────────────────────────────────
pub async fn buscar(pool: &PgPool, presupuesto: i32, texto: &str) -> Result<Vec<PartidaBuscada>, ReturnCode> {
    dal::buscar(pool, presupuesto, texto).await
}
