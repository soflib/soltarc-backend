// Programa...: domain::models::catalog_g
// Descripción: Modelo del catálogo general
//
// Mapea exactamente las columnas que devuelven las funciones
// sp_cpa_catalogo_qry / lst_all / qry_tipo / lst_tipos
//
// Postgres devuelve columnas de TABLE(...) como nullable — por eso
// todos los campos excepto id son Option<T>

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ─────────────────────────────────────────────
// Modelo DAL — usado en query_as! 
// Campos Option porque Postgres TABLE() devuelve nullable
// ─────────────────────────────────────────────
#[derive(Debug, sqlx::FromRow)]
pub struct CatalogG {
    pub id:          Option<i32>,
    pub tipo:        Option<i16>,
    pub nombre:      Option<String>,
    pub activo:      Option<bool>,
    pub comentarios: Option<String>,
}

// ─────────────────────────────────────────────
// DTO entrada — Alta y Cambios (HTTP handler)
// id es Option porque Alta no lo envía
// ─────────────────────────────────────────────
#[derive(Debug, Deserialize, ToSchema)]
pub struct CatalogGInput {
    pub id:          Option<i32>,
    pub tipo:        i32,
    pub nombre:      String,
    pub activo:      bool,
    pub comentarios: Option<String>,
}

// ─────────────────────────────────────────────
// DTO salida — respuestas HTTP
// ─────────────────────────────────────────────
#[derive(Debug, Serialize, ToSchema)]
pub struct CatalogGOutput {
    pub id:          i32,
    pub tipo:        i32,
    pub nombre:      String,
    pub activo:      bool,
    pub comentarios: Option<String>,
}

// ─────────────────────────────────────────────
// Conversión DAL → DTO salida
// ─────────────────────────────────────────────
impl From<CatalogG> for CatalogGOutput {
    fn from(c: CatalogG) -> Self {
        CatalogGOutput {
            id:          c.id.unwrap_or(0),
            tipo:        c.tipo.unwrap_or(0) as i32,
            nombre:      c.nombre.unwrap_or_default(),
            activo:      c.activo.unwrap_or(false),
            comentarios: c.comentarios,
        }
    }
}