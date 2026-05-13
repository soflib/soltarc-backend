// Programa...: DetalleProyectos
// Descripción: Tabla cpa_DetalleProyectos
// Origen.....: eDetalleProyectos.cs
//
// NOTA: En C# usaba System.Data.Entity.Hierarchy.HierarchyId para el campo
// Nodo. SQL Server lo soporta nativamente. En Rust se representa como String
// ya que sqlx no tiene soporte nativo para HierarchyId — se lee y escribe
// como texto (ej: "/1/2/3/") y se parsea en la aplicación si se necesita.

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct DetalleProyectos {
    pub id: i32,
    pub proyecto: i32,
    pub tipo: i32,
    pub secuencia: i32,
    pub descripcion: String,
    pub comentarios: String,
    #[schema(value_type = f64)]
    pub presupuesto: Decimal,
    pub fecha_inicio: NaiveDateTime,
    pub fecha_fin: NaiveDateTime,
    pub fecha_termina: NaiveDateTime,
    pub estado: i32,
    pub nodo: String, // HierarchyId de SQL Server — se maneja como String
}
