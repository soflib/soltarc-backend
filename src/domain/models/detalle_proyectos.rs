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
use serde::Serialize;
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

// ─────────────────────────────────────────────
// NodoArbol
// Origen: sp_cpa_detalleproy_arbol(proyecto) — `ruta` viene como NULL
//         sp_cpa_detalleproy_buscar(proyecto, texto) — `ruta` viene con valor
//
// Un solo struct para ambos endpoints: el árbol (sin ruta) se obtiene
// agregando "NULL::TEXT AS ruta" al SELECT del DAL.
// ─────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct NodoArbol {
    pub id:          i32,
    pub nodo:        String,
    pub descripcion: String,
    pub nivel:       i32,
    #[schema(value_type = f64)]
    pub importe:     Decimal,
    pub estado:      i32,
    /// Ruta ancestral ("padre > hijo > nieto"). None cuando viene del árbol completo.
    pub ruta:        Option<String>,
}
