// Programa...: handler::sistema::contacto
// Descripción: Formulario público "Contáctanos" del sitio (landing) → manda el
//              mensaje a la casilla de contacto vía Outlook/Graph
//              (infrastructure::email, la MISMA app de Azure de este backend).
//
// Ruta PÚBLICA (sin auth): la consume el sitio de marketing, no el dashboard.
// El destino es CONTACT_EMAIL_ADDRESS (default customer-service@soltarc.com).

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{info, warn};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContactoInput {
    pub nombre:  String,
    pub email:   String,
    pub mensaje: String,
}

/// Escapa lo mínimo para meter texto del visitante en el HTML del correo sin
/// permitir inyección de marcado.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[utoipa::path(
    post,
    path = "/contacto",
    request_body = ContactoInput,
    responses(
        (status = 200, description = "Mensaje de contacto enviado", body = Value),
        (status = 400, description = "Datos inválidos",             body = Value),
        (status = 502, description = "No se pudo enviar el correo", body = Value),
    ),
    tag = "Contacto"
)]
pub async fn contacto(Json(body): Json<ContactoInput>) -> (StatusCode, Json<Value>) {
    let nombre  = body.nombre.trim().to_string();
    let email   = body.email.trim().to_string();
    let mensaje = body.mensaje.trim().to_string();

    if nombre.is_empty() || email.is_empty() || mensaje.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "mensaje": "Nombre, correo y mensaje son requeridos." })),
        );
    }

    // Casilla destino: env CONTACT_EMAIL_ADDRESS o, por defecto, la de soporte.
    let to = std::env::var("CONTACT_EMAIL_ADDRESS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "customer-service@soltarc.com".to_string());

    let (e_nombre, e_email, e_mensaje) = (esc(&nombre), esc(&email), esc(&mensaje));
    let vars = [
        ("nombre",  e_nombre.as_str()),
        ("email",   e_email.as_str()),
        ("mensaje", e_mensaje.as_str()),
    ];

    match crate::infrastructure::email::outlook::send_template(&to, "contact", &vars).await {
        Ok(_) => {
            info!(%to, %email, "contacto: mensaje del sitio enviado");
            (StatusCode::OK, Json(json!({ "mensaje": "Mensaje enviado correctamente." })))
        }
        Err(e) => {
            warn!(error = %e, "contacto: envío de correo falló");
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "mensaje": "No se pudo enviar el mensaje. Inténtalo más tarde." })),
            )
        }
    }
}
