// Programa...: CentrosCosto
// Descripción: Tabla Centros de Costo
// Origen.....: eCentrosCosto.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct CentrosCosto {
    pub id: i32,
    pub nombre: String,
    pub tipo: i32,
    pub comentarios: String,
    pub activo: bool,
}
