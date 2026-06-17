// Plantillas de correo en JSON.
//
// Se EMBEBEN en el binario con include_str! (fuente: archlib-backend/templates/emails.json),
// así SIEMPRE están disponibles sin depender del working dir ni del deploy (Docker, etc.).
// Para editar la plantilla, cambia el JSON y recompila.
//
// Override opcional en runtime: si se define `EMAIL_TEMPLATES_PATH` y el archivo existe,
// se usa ese en vez de la copia embebida (permite cambiar el correo sin recompilar).
//
// Formato: { "<clave>": { "subject": "...", "html": "...{{var}}..." }, ... }

use serde::Deserialize;
use std::collections::HashMap;

/// Copia embebida en el binario.
const EMBEDDED: &str = include_str!("../../../templates/emails.json");

#[derive(Debug, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub html:    String,
    /// Ruta opcional a un archivo para adjuntar (p.ej. un PDF). Best-effort:
    /// si no se puede leer al enviar, el correo sale sin adjunto.
    #[serde(default)]
    pub attachment: Option<String>,
}

/// Carga la plantilla `clave`. Usa el archivo de `EMAIL_TEMPLATES_PATH` si está
/// definido y se puede leer; si no, la copia embebida.
pub fn load(clave: &str) -> Result<EmailTemplate, String> {
    let raw: String = match std::env::var("EMAIL_TEMPLATES_PATH") {
        Ok(p) if !p.trim().is_empty() =>
            std::fs::read_to_string(&p).unwrap_or_else(|_| EMBEDDED.to_string()),
        _ => EMBEDDED.to_string(),
    };
    let mut map: HashMap<String, EmailTemplate> = serde_json::from_str(&raw)
        .map_err(|e| format!("JSON de plantillas inválido: {e}"))?;
    map.remove(clave)
        .ok_or_else(|| format!("plantilla '{clave}' no existe"))
}

/// Sustituye cada placeholder `{{k}}` por su valor.
pub fn render(s: &str, vars: &[(&str, &str)]) -> String {
    let mut out = s.to_string();
    for (k, v) in vars {
        out = out.replace(&["{{", k, "}}"].concat(), v);
    }
    out
}
