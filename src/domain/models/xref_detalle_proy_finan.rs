// Programa...: XrefDetalleProyFinan
// Descripción: Tabla de referencia cruzada gasto partidas
// Origen.....: eXref_DetalleProy_Finan.cs

use rust_decimal::Decimal;
use serde::Serialize;
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

// Saldo de un egreso: monto original vs lo aplicado en xrefs.
// Devuelto por sp_cpa_xref_saldo(transaccion). Usado para mostrar al usuario
// cuánto puede aplicar antes de exceder el egreso.
#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct XrefSaldo {
    #[schema(value_type = f64)]
    pub monto_egreso: Decimal,
    #[schema(value_type = f64)]
    pub aplicado:     Decimal,
    #[schema(value_type = f64)]
    pub disponible:   Decimal,
}
