pub mod account_statement;
pub mod dashboard;
pub mod expense_detail;
pub mod project_images;
pub mod project_tasks;
pub mod project_tree;
pub mod weekly_plan;
pub mod work_progress;

use std::collections::HashMap;
use sqlx::PgPool;
use uuid::Uuid;

// Tipos de cpa_catalogos usados por el portal para resolver FKs numéricos a nombre.
pub const CAT_ESTADO_PROYECTOS: i32 = 1; // estado de un proyecto
pub const CAT_ESTADO_PARTIDAS: i32  = 8; // estado de una tarea/partida

/// Mapa id→nombre de un catálogo (cpa_catalogos) por `tipo`, para el tenant dado.
/// Resuelve FKs numéricos (estado, tipo de tarea, …) a su nombre legible desde
/// los handlers del portal, que NO pueden llamar `/general/catalogs` (requiere
/// rol Arquitecto). Si el catálogo falla, devuelve un mapa vacío y el frontend
/// cae al número crudo.
pub async fn catalogo_map(pool: &PgPool, tipo: i32, tenant_id: Uuid) -> HashMap<i32, String> {
    match crate::dal::catalog_g::obtiene_por_tipo(pool, tipo, tenant_id).await {
        Ok(cats) => cats
            .into_iter()
            .filter_map(|c| match (c.id, c.nombre) {
                (Some(id), Some(nombre)) => Some((id, nombre)),
                _ => None,
            })
            .collect(),
        Err(_) => HashMap::new(),
    }
}
