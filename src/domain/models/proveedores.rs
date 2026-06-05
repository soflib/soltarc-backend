// Programa...: Proveedores
// Descripción: Tabla cpa_Proveedores
// Origen.....: eProveedores.cs
//
// PATRÓN _nombre: tipo_nombre y giro_nombre son Option<String> poblados por
// LEFT JOIN en los SPs _qry y _lstall. Son None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use utoipa::ToSchema;
use uuid::Uuid;

// tenant_id semántica:
//   None       → plantilla GLOBAL (no se puede editar/borrar desde la app)
//   Some(uuid) → proveedor PRIVADO del tenant
#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Proveedores {
    pub id: Option<i32>,
    pub nombre: String,
    pub contacto: String,
    pub direccion: String,
    pub telefono: String,
    pub mail: String,
    pub cuenta_banco: String,
    pub tipo: i32,                   // FK → cpa_catalogos tipo=3 (Tipo Persona moral)
    pub tipo_nombre: Option<String>, // resuelto por SP vía LEFT JOIN
    pub giro: i32,                   // FK → cpa_catalogos tipo=4 (Giro/Tipo proveedor)
    pub giro_nombre: Option<String>, // resuelto por SP vía LEFT JOIN
    pub comentarios: String,
    pub activo: bool,
    pub rfc: String,
    pub tenant_id: Option<Uuid>,     // None = global; Some = privado del tenant
}
