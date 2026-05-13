// Programa...: SaldosBancos
// Descripción: Tabla cpa_SaldosBancos
// Origen.....: eSaldosBancos.cs
//
// PATRÓN _nombre: banco_nombre es Option<String> poblado por LEFT JOIN
// en los SPs _qry, _lstbco y _lstall. Es None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct SaldosBancos {
    pub id: Option<i32>,
    pub banco: i32,                   // FK → cpa_catalogos tipo=5 (Bancos)
    pub banco_nombre: Option<String>, // resuelto por SP vía LEFT JOIN
    pub ano: i32,
    pub mes: i32,
    #[schema(value_type = f64)]
    pub monto: Decimal,
}
