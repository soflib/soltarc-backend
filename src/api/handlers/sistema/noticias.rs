// Programa...: handler::sistema::noticias
// Descripción: Noticias / próximos cambios del producto que se muestran en el
//   dashboard (sección "Noticias", después de Soporte). Lista GLOBAL (no por
//   tenant) servida desde el backend para poder cambiar "los siguientes pasos"
//   sin tocar el frontend.
//
// Rutas:
//   GET /sistema/noticias → { noticias: [ Noticia… ] }   (cualquier usuario autenticado)
//
// ⭑ Por ahora la lista vive aquí como dato estático: para cambiar lo que ve el
//   dashboard, EDITA `NOTICIAS` abajo. El día que se quiera editar sin redeploy,
//   mover la lista a una tabla y leerla en `listar` (la firma/JSON no cambian).

use axum::{http::StatusCode, Json};
use serde::Serialize;
use serde_json::{json, Value};

/// Una noticia / próximo cambio del producto. Bilingüe: el dashboard elige el
/// texto según el idioma de la interfaz (ES/EN). El `estado` se traduce en el
/// front (clave i18n `news.badge.*`), por eso aquí va como código.
#[derive(Serialize)]
pub struct Noticia {
    /// Identificador estable de la noticia.
    pub id: u32,
    /// Título (español).
    pub titulo: &'static str,
    /// Título (inglés).
    pub titulo_en: &'static str,
    /// Detalle opcional ES (cadena vacía si no aplica).
    pub descripcion: &'static str,
    /// Detalle opcional EN (cadena vacía si no aplica).
    pub descripcion_en: &'static str,
    /// Estado: "proximamente" | "en_progreso" | "lanzado".
    pub estado: &'static str,
    /// Orden de despliegue (menor primero).
    pub orden: u32,
}

/// ⭑ EDITA AQUÍ los próximos pasos que verá el dashboard (ambos idiomas).
const NOTICIAS: &[Noticia] = &[
    Noticia {
        id: 1,
        titulo:    "Base de datos privadas",
        titulo_en: "Private databases",
        descripcion: "",
        descripcion_en: "",
        estado: "proximamente",
        orden: 1,
    },
    Noticia {
        id: 2,
        titulo:    "Notificación en tiempo real a tus clientes ante cualquier cambio o terminación del proyecto",
        titulo_en: "Real-time notifications to your clients on any change or project completion",
        descripcion: "",
        descripcion_en: "",
        estado: "proximamente",
        orden: 2,
    },
    Noticia {
        id: 3,
        titulo:    "Trabajando en tu seguridad: buscamos la certificación ISO 27001",
        titulo_en: "Working on your security: pursuing ISO 27001 certification",
        descripcion: "",
        descripcion_en: "",
        estado: "proximamente",
        orden: 3,
    },
];

/// GET /sistema/noticias → `{ "noticias": [ { id, titulo, descripcion, estado, orden } ] }`.
/// Lista ordenada por `orden`. Cualquier usuario autenticado.
pub async fn listar() -> (StatusCode, Json<Value>) {
    let mut noticias: Vec<&Noticia> = NOTICIAS.iter().collect();
    noticias.sort_by_key(|n| n.orden);
    (StatusCode::OK, Json(json!({ "noticias": noticias })))
}
