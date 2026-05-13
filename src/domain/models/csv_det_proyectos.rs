// Programa...: CsvDetProyectos
// Descripción: Archivo CSV para detalle de proyectos
// Origen.....: CSVDetProyectos.cs

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
pub struct CsvDetProyectos {
    pub proyecto: i32,
    pub tipo: i32,
    pub secuencia: i32,
    pub descripcion: String,
    pub comentarios: String,
    #[schema(value_type = f64)]
    pub presupuesto: Decimal,
    pub fecha_inicio: NaiveDateTime,
    pub fecha_fin: NaiveDateTime,
    pub estado: i32,
    pub nodo: String,
}
