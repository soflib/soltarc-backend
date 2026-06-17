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
// Espacio del tenant (cuenta para su cuota de 25GB), clasificado por área de obra:
//   {tenant_id}/proyectos/{proyecto_id}/{area}/{uuid}_{archivo}
// Soporte (capturas de errores; FUERA del espacio/cuota del tenant):
//   support/{tenant_id}/{timestamp}/{archivo}

/// Áreas/etapas de obra válidas para clasificar los archivos de un proyecto.
pub const AREAS_OBRA: [&str; 5] = ["terreno", "obra_negra", "obra_gris", "acabados", "final"];

/// Slug usado cuando el área recibida no es válida o la key es antigua (sin área).
pub const AREA_DEFAULT: &str = "sin_clasificar";

/// Sanitiza un nombre de archivo para usarlo en una key S3 (sin '/', espacios → '_').
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Normaliza el área recibida del cliente: si no es una de las válidas → AREA_DEFAULT.
pub fn normalize_area(area: &str) -> String {
    let a = area.trim().to_lowercase();
    if AREAS_OBRA.contains(&a.as_str()) { a } else { AREA_DEFAULT.to_string() }
}

/// Extrae el área desde una key de proyecto:
///   {tenant}/proyectos/{id}/{area}/{archivo}  →  {area}
/// Keys antiguas sin segmento de área → AREA_DEFAULT.
pub fn area_from_key(key: &str) -> String {
    let parts: Vec<&str> = key.split('/').collect();
    if parts.len() >= 5 && parts[1] == "proyectos" {
        parts[3].to_string()
    } else {
        AREA_DEFAULT.to_string()
    }
}

/// Archivo de un proyecto dentro del espacio del tenant, bajo un área de obra
/// (uuid evita colisiones). `area` debe venir ya normalizada (normalize_area).
pub fn tenant_proyecto_file(tenant_id: &uuid::Uuid, proyecto_id: i32, area: &str, filename: &str) -> String {
    let safe = sanitize_filename(filename);
    let uid = uuid::Uuid::new_v4().simple();
    format!("{tenant_id}/proyectos/{proyecto_id}/{area}/{uid}_{safe}")
}

/// Prefijo de TODO el espacio del tenant.
pub fn tenant_prefix(tenant_id: &uuid::Uuid) -> String {
    format!("{tenant_id}/")
}

/// Logo del tenant (key fija; se sobrescribe al actualizar). Se usa en la pantalla
/// de Configuración y para insertarlo en los reportes PDF. El content-type se fija
/// al subir, así que la key no necesita extensión.
pub fn tenant_logo(tenant_id: &uuid::Uuid) -> String {
    format!("{tenant_id}/config/logo")
}

/// Captura de soporte: support/{tenant}/{ts}/{archivo}.
pub fn support_file(tenant_id: &uuid::Uuid, ts: &str, filename: &str) -> String {
    let safe = sanitize_filename(filename);
    format!("support/{tenant_id}/{ts}/{safe}")
}
