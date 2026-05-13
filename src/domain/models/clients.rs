// Programa...: Clientes
// Descripción: Tabla cpa_Clientes
// Origen.....: eClientes.cs
//
// PATRÓN _nombre: tipo_nombre es Option<String> poblado por LEFT JOIN
// en los SPs _qry y _lstact. Es None en escrituras (alta/cambios).
// Ver PATRON_NOMBRES.txt en la raíz del proyecto.

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Clientes {
    pub id: i32,
    pub nombre: String,
    pub direccion: String,
    pub telefono: String,
    pub mail: String,
    pub cuenta_banco: String,
    pub comentarios: String,
    pub tipo: i32,                   // FK → cpa_catalogos tipo=3 (Tipo Persona moral)
    pub tipo_nombre: Option<String>, // resuelto por SP vía LEFT JOIN
    pub activo: bool,
    pub condiciones: String,
}
