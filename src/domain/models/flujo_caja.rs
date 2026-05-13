// Programa...: Flujo
// Descripción: Flujo de caja
// Origen.....: eFlujo.cs

use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct FlujoCaja {
    pub tipo: String,
    pub banco: i32,
    #[schema(value_type = f64)]
    pub monto: Decimal,
}
