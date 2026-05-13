// Programa...: CostosEstimados
// Descripción: Tabla Costos Estimados
// Origen.....: eCostosEstimados.cs

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct CostosEstimados {
    pub id: i32,
    pub tipo: i32,
    pub nombre: String,
    pub descripcion: String,
    pub unidad: i32,
    pub fecha:       NaiveDateTime,
    #[schema(value_type = f64)]
    pub importe:     Decimal,
    pub activo:      bool,
}
