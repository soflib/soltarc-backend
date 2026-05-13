// Programa...: Egresos
// Descripción: Tabla cpa_Egresos
// Origen.....: eEgresos.cs
//
// PATRÓN _nombre: los campos *_nombre son Option<String> poblados por LEFT JOIN
// en los SPs _qry y _lstall. Son None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use uuid::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Egresos {
    pub id: Option<i32>,
    pub fecha: NaiveDateTime,
    pub banco: i32,                          // FK → cpa_catalogos tipo=5 (Bancos)
    pub banco_nombre: Option<String>,        // resuelto por SP vía LEFT JOIN
    pub cuenta: String,
    pub forma_pago: String,
    pub centro_costo: i32,                   // FK → cpa_centroscosto
    pub centro_costo_nombre: Option<String>, // resuelto por SP vía LEFT JOIN
    #[schema(value_type = f64)]
    pub monto: Decimal,
    pub referencia: String,
    pub comentario: String,
    pub fecha_aplica: NaiveDateTime,
    pub proyecto: i32,                       // FK → cpa_proyectos
    pub proyecto_nombre: Option<String>,     // resuelto por SP vía LEFT JOIN
    pub proveedor: i32,                      // FK → cpa_proveedores
    pub proveedor_nombre: Option<String>,    // resuelto por SP vía LEFT JOIN
    pub usuario: Uuid,
}
