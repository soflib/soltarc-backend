// Programa...: AccesosRapidos
// Descripción: Tabla de accesos rápidos
// Origen.....: eAccesosRapidos.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct AccesosRapidos {
    pub id: i32,
    pub funcion: String,
    pub tool_tip: String,
    pub imagen: String,
}
