// Key conventions for Contabo object storage.
//
// All project files live under  proyectos/{proyecto_id}/
// Budget documents live under   presupuestos/{ppto_id}/
//
// Bucket layout:
//   proyectos/{id}/fotos/          ← progress photos
//   proyectos/{id}/planos/         ← architectural plans (PDF, DWG, etc.)
//   proyectos/{id}/contratos/      ← contracts
//   proyectos/{id}/egresos/{id}/   ← expense invoices / receipts
//   proyectos/{id}/ingresos/{id}/  ← income invoices / receipts
//   presupuestos/{id}/cotizaciones/ ← quotes / cotizaciones

/// Progress photo for a project.
pub fn foto_progreso(proyecto_id: i32, filename: &str) -> String {
    format!("proyectos/{proyecto_id}/fotos/{filename}")
}

/// Architectural plan or drawing for a project.
pub fn plano(proyecto_id: i32, filename: &str) -> String {
    format!("proyectos/{proyecto_id}/planos/{filename}")
}

/// Contract document for a project.
pub fn contrato(proyecto_id: i32, filename: &str) -> String {
    format!("proyectos/{proyecto_id}/contratos/{filename}")
}

/// Expense invoice or payment receipt for a specific egreso.
pub fn factura_egreso(proyecto_id: i32, egreso_id: i32, filename: &str) -> String {
    format!("proyectos/{proyecto_id}/egresos/{egreso_id}/{filename}")
}

/// Income invoice or payment receipt for a specific ingreso.
pub fn factura_ingreso(proyecto_id: i32, ingreso_id: i32, filename: &str) -> String {
    format!("proyectos/{proyecto_id}/ingresos/{ingreso_id}/{filename}")
}

/// Quote / cotización document attached to a budget (presupuesto).
pub fn cotizacion(ppto_id: i32, filename: &str) -> String {
    format!("presupuestos/{ppto_id}/cotizaciones/{filename}")
}

/// List prefix for all files belonging to a project.
pub fn proyecto_prefix(proyecto_id: i32) -> String {
    format!("proyectos/{proyecto_id}/")
}

/// List prefix for all photos in a project.
pub fn fotos_prefix(proyecto_id: i32) -> String {
    format!("proyectos/{proyecto_id}/fotos/")
}
