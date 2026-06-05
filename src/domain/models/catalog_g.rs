// Programa...: domain::models::catalog_g
// Descripción: Modelo del catálogo general (multi-tenant)
//
// Mapea exactamente las columnas que devuelven las funciones
// sp_cpa_catalogo_qry / lst_all / qry_tipo / lst_tipos.
//
// Postgres devuelve columnas de TABLE(...) como nullable — por eso
// todos los campos excepto id son Option<T>.
//
// tenant_id semántica:
//   None       → catálogo GLOBAL del sistema (todos los tenants lo ven,
//                no se puede editar/borrar desde la app)
//   Some(uuid) → catálogo PRIVADO del tenant indicado

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
    pub tenant_id:   Option<Uuid>,
}

// ─────────────────────────────────────────────
// DTO entrada — Alta y Cambios (HTTP handler)
// id es Option porque Alta no lo envía.
// tenant_id NO viene del body: lo inyecta el handler desde el JWT.
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
// tenant_id se incluye para que la UI distinga catálogos globales (null)
// de los propios del tenant.
// ─────────────────────────────────────────────
#[derive(Debug, Serialize, ToSchema)]
pub struct CatalogGOutput {
    pub id:          i32,
    pub tipo:        i32,
    pub nombre:      String,
    pub activo:      bool,
    pub comentarios: Option<String>,
    pub tenant_id:   Option<String>,
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
            tenant_id:   c.tenant_id.map(|u| u.to_string()),
        }
    }
}
