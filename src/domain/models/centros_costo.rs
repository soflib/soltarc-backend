// Programa...: CentrosCosto
// Descripción: Tabla Centros de Costo
// Origen.....: eCentrosCosto.cs

use utoipa::ToSchema;
use uuid::Uuid;

// tenant_id semántica:
//   None       → registro GLOBAL del sistema (no editable desde la app)
//   Some(uuid) → registro PRIVADO del tenant
#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct CentrosCosto {
    pub id: i32,
    pub nombre: String,
    pub tipo: i32,
    pub comentarios: String,
    pub activo: bool,
    pub tenant_id: Option<Uuid>,
}
