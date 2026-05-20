// Programa...: Egresos
// Descripción: Tabla cpa_Egresos
// Origen.....: eEgresos.cs
//
// PATRÓN _nombre: los campos *_nombre son Option<String> poblados por LEFT JOIN
// en los SPs _qry y _lstall. Son None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use chrono::{NaiveDate, NaiveDateTime};
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

// ─────────────────────────────────────────────
// Búsqueda paginada (sp_cpa_egresos_search)
// Total_count = COUNT(*) OVER () repetido en cada fila.
// ─────────────────────────────────────────────
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EgresoConTotal {
    #[sqlx(flatten)]
    pub egreso:      Egresos,
    pub total_count: i64,
}

#[derive(Debug, Default, Clone)]
pub struct EgresosFilter {
    pub proyecto:     Option<i32>,
    pub proveedor:    Option<i32>,
    pub centro_costo: Option<i32>,
    pub fecha_ini:    Option<NaiveDate>,
    pub fecha_fin:    Option<NaiveDate>,
    pub q:            Option<String>,
    pub offset:       i32,
    pub limit:        i32,
}
