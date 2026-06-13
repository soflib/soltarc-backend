// Programa...: Proyectos
// Descripción: Tabla cpa_Proyectos
// Origen.....: eProyectos.cs

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use utoipa::ToSchema;
use uuid::Uuid;

// tenant_id semántica:
//   None       → no debería ocurrir en proyectos (no hay proyectos globales)
//   Some(uuid) → proyecto PRIVADO del tenant
#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Proyectos {
    pub id: i32,
    pub tipo: i32,
    pub nombre: String,
    pub descripcion: String,
    pub direccion: String,
    pub comentarios: String,
    pub estado: i32,
    #[schema(value_type = f64)]
    pub presupuesto: Decimal,
    pub fecha_ini: NaiveDateTime,
    pub fecha_fin: NaiveDateTime,
    pub asignado: String,
    pub cliente: i32,
    pub activo: bool,
    pub gn_id: i32,
    pub gn_usr_id: i32,
    pub dir_imagenes: String,
    pub tenant_id: Option<Uuid>,
}

// Par (grupo, usuario) asignado a un proyecto (cpa_proyecto_asignaciones).
// gn_usr_id = 0 → "todo el grupo". usuario_user_id es el email en gn_usuarios
// (el handler lo resuelve a nombre completo vía gRPC al servicio de auth).
#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct ProyectoAsignacion {
    pub gn_id: i32,
    pub gn_usr_id: i32,
    pub grupo_nombre: String,
    pub usuario_user_id: String,
}
