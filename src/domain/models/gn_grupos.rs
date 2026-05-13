// Programa...: GnGrupos
// Descripción: Tabla de grupos de negocio
// Origen.....: eGNGrupos.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct GnGrupos {
    pub id: i32,
    pub nombre: String,
    pub descripcion: String,
    pub activo: bool,
}
