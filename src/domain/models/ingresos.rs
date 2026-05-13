// Programa...: Ingresos
// Descripción: Tabla cpa_Ingresos
// Origen.....: eIngresos.cs
//
// PATRÓN _nombre: los campos *_nombre son Option<String> poblados por LEFT JOIN
// en los SPs _qry y _lstall. Son None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Ingresos {
    pub id: Option<i32>,
    pub fecha: NaiveDateTime,
    pub banco: i32,                      // FK → cpa_catalogos tipo=5 (Bancos)
    pub banco_nombre: Option<String>,    // resuelto por SP vía LEFT JOIN
    pub cuenta: String,
    pub forma_pago: String,
    pub proyecto: i32,                   // FK → cpa_proyectos
    pub proyecto_nombre: Option<String>, // resuelto por SP vía LEFT JOIN
    #[schema(value_type = f64)]
    pub monto: Decimal,
    pub referencia: String,
    pub comentario: String,
    pub fecha_aplica: NaiveDateTime,
    pub cliente: i32,                    // FK → cpa_clientes
    pub cliente_nombre: Option<String>,  // resuelto por SP vía LEFT JOIN
    pub usuario_ms: Uuid,
}
