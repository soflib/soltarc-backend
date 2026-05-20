// Programa...: Ingresos
// Descripción: Tabla cpa_Ingresos
// Origen.....: eIngresos.cs
//
// PATRÓN _nombre: los campos *_nombre son Option<String> poblados por LEFT JOIN
// en los SPs _qry y _lstall. Son None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use chrono::{NaiveDate, NaiveDateTime};
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

// ─────────────────────────────────────────────
// Búsqueda paginada (sp_cpa_ingresos_search)
// El SP devuelve una columna extra `total_count` con COUNT(*) OVER ()
// repetido en cada fila para evitar un segundo viaje al servidor.
// ─────────────────────────────────────────────
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IngresoConTotal {
    #[sqlx(flatten)]
    pub ingreso:     Ingresos,
    pub total_count: i64,
}

#[derive(Debug, Default, Clone)]
pub struct IngresosFilter {
    pub proyecto:  Option<i32>,
    pub cliente:   Option<i32>,
    pub fecha_ini: Option<NaiveDate>,
    pub fecha_fin: Option<NaiveDate>,
    pub q:         Option<String>,
    pub offset:    i32,
    pub limit:     i32,
}
