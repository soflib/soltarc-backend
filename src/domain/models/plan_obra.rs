// Programa...: PlanObra
// Descripción: Plan de obra
// Origen.....: ePlanObra.cs

use chrono::NaiveDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct PlanObra {
    pub id: i32,
    pub fecha_ini: NaiveDateTime,
    pub fecha_fin: NaiveDateTime,
    pub estado: i32,
    pub comentarios: String,
    pub fecha_termina: NaiveDateTime,
}
