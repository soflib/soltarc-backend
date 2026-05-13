// Programa...: Seguridad
// Descripción: Variables de seguridad del sistema
// Origen.....: eSeguridad.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Seguridad {
    pub gpo_id: i32,
    pub gpo_nombre: String,
    pub gpo_descrip: String,
    pub gpo_activo: bool,
    pub usr_id: i32,
    pub usr_nivel: i32,
    pub usr_activo: bool,
}
