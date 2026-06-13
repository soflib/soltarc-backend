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

// ── Multi-tenant layout (actual) ─────────────────────────────────────────────
// Espacio del tenant (cuenta para su cuota de 25GB):
//   t/{tenant_id}/proyectos/{proyecto_id}/{uuid}_{archivo}
// Soporte (capturas de errores; FUERA del espacio/cuota del tenant):
//   support/{tenant_id}/{timestamp}/{archivo}

/// Sanitiza un nombre de archivo para usarlo en una key S3 (sin '/', espacios → '_').
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Archivo de un proyecto dentro del espacio del tenant (uuid evita colisiones).
pub fn tenant_proyecto_file(tenant_id: &uuid::Uuid, proyecto_id: i32, filename: &str) -> String {
    let safe = sanitize_filename(filename);
    let uid = uuid::Uuid::new_v4().simple();
    format!("t/{tenant_id}/proyectos/{proyecto_id}/{uid}_{safe}")
}

/// Prefijo de TODO el espacio del tenant.
pub fn tenant_prefix(tenant_id: &uuid::Uuid) -> String {
    format!("t/{tenant_id}/")
}

/// Captura de soporte: support/{tenant}/{ts}/{archivo}.
pub fn support_file(tenant_id: &uuid::Uuid, ts: &str, filename: &str) -> String {
    let safe = sanitize_filename(filename);
    format!("support/{tenant_id}/{ts}/{safe}")
}
