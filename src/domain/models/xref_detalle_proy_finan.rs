// Programa...: XrefDetalleProyFinan
// Descripción: Tabla de referencia cruzada gasto partidas
// Origen.....: eXref_DetalleProy_Finan.cs

use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct XrefDetalleProyFinan {
    pub id: i32,
    pub partida: i32,
    pub tipo: i32,
    pub transaccion: i32,
    pub comentario: String,
    pub proyecto: i32,
    #[schema(value_type = f64)]
    pub monto_aplica: Decimal,
}
