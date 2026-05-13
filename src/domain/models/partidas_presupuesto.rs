// Programa...: PartidasPresupuesto
// Descripción: Tabla partidas presupuestales
// Origen.....: ePartidasPresupuesto.cs

use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
pub struct PartidasPresupuesto {
    pub id: i32,
    pub presupuesto: i32,
    pub nodo: String,
    pub concepto: String,
    pub unidad: i32,
    #[schema(value_type = f64)]
    pub cantidad: Decimal,
    #[schema(value_type = f64)]
    pub precio_u: Decimal,
    pub secuencia: i32,
    pub nivel: i32,
    pub calculo: i32,
}
