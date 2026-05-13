// Programa...: Unidades
// Descripción: Tabla de unidades de medida
// Origen.....: eUnidades.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Unidades {
    pub id: Option<i32>,
    pub tipo: i32,
    pub descripcion: String,
    pub nombre_corto: String,
    pub activa: bool,
}
