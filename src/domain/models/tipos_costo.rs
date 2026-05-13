// Programa...: TiposCosto
// Descripción: Tabla de tipos de costo
// Origen.....: eTiposCosto.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct TiposCosto {
    pub id: Option<i32>,
    pub nombre: String,
    pub descripcion: String,
    pub activo: bool,
    pub imagen: String,
}
