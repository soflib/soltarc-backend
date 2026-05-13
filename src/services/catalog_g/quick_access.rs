// Programa...: accesos_rapidos
// Descripción: Mantenimiento a los botones de acceso rápido
// Origen.....: AccesosRapidos.aspx.cs
//
// DAL que usa:
//   crate::dal::configura::accesos_rapidos_upd  → cambios
//   crate::dal::configura::accesos_rapidos_qry  → consulta

use crate::dal::configura;
use crate::domain::models::accesos_rapidos::AccesosRapidos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// LISTA TODOS — AccesosRapidosLSTAll
// ─────────────────────────────────────────────
pub async fn lista_todos(pool: &PgPool) -> Result<Vec<AccesosRapidos>, ReturnCode> {
    configura::accesos_rapidos_lst_all(pool).await
}

// ─────────────────────────────────────────────
// CAMBIOS — AccesosRapidosUPD
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, ar: &AccesosRapidos) -> ReturnCode {
    configura::accesos_rapidos_upd(pool, ar).await
}

// ─────────────────────────────────────────────
// CONSULTA — AccesosRapidosQRY
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<AccesosRapidos>, ReturnCode> {
    configura::accesos_rapidos_qry(pool, id).await
}
