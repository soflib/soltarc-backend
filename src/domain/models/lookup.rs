// Programa...: models/lookup.rs
// Descripción: Tipos genéricos para autocomplete y paginación.
//
// LookupItem  → fila ligera (id + etiqueta) que devuelven los SPs *_lookup.
// PageOf<T>   → envoltura paginada que arman los handlers a partir de los
//               SPs *_search (los SPs devuelven cada fila con total_count).

use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct LookupItem {
    pub id:       i32,
    pub etiqueta: String,
}

#[derive(Debug, Serialize)]
pub struct PageOf<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page:  i32,
    pub size:  i32,
}
