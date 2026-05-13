// Programa...: services::clients::project_tree
// Descripción: Árbol de tareas del proyecto para el portal de clientes
// Origen.....: Cte_ArbolProyecto.aspx.cs — ArbolTareasProyecto

use crate::dal::reportes;
use crate::domain::models::reportes::PartidasArbol;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn arbol_tareas(pool: &PgPool, proyecto: i32) -> Result<Vec<PartidasArbol>, ReturnCode> {
    reportes::arbol_tareas_proyecto(pool, proyecto).await
}
