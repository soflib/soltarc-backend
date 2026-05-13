// Programa...: Proyectos
// Descripción: Tabla cpa_Proyectos
// Origen.....: eProyectos.cs

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use utoipa::ToSchema;

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
}
