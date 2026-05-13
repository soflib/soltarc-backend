// Programa...: handler::sistema::configura
// Origen.....: oConfigura.cs
//
// Rutas:
//   PUT /sistema/configura    → cambia_configuracion
//   GET /sistema/configura    → carga_configuracion

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, error, info};
use utoipa::ToSchema;

use crate::domain::models::configura::Configura;
use crate::infrastructure::db::app_state::AppState;
use crate::services::sistema::configura as svc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConfiguraInput {
    pub nom_empresa:     String,
    pub tipo_unidad:     String,
    pub image_path:      String,
    pub num_rens_ppto:   i32,
    pub i_top:           i32,
    pub i_rig:           i32,
    pub i_bot:           i32,
    pub i_lef:           i32,
    pub ppto_color_edit: String,
    pub color_nivel1:    String,
    pub color_nivel2:    String,
    pub color_nivel3:    String,
    pub color_nivel4:    String,
    pub i_dias_previos:  i32,
    pub num_rens_proy:   i32,
    pub num_rens_otros:  i32,
    pub fin_tarea:       i32,
    pub pag_ancho_total: i32,
    pub largo_concepto:  i32,
}

fn to_model(b: ConfiguraInput) -> Configura {
    Configura {
        nom_empresa:     b.nom_empresa,
        tipo_unidad:     b.tipo_unidad,
        image_path:      b.image_path,
        num_rens_ppto:   b.num_rens_ppto,
        i_top:           b.i_top,
        i_rig:           b.i_rig,
        i_bot:           b.i_bot,
        i_lef:           b.i_lef,
        ppto_color_edit: b.ppto_color_edit,
        color_nivel1:    b.color_nivel1,
        color_nivel2:    b.color_nivel2,
        color_nivel3:    b.color_nivel3,
        color_nivel4:    b.color_nivel4,
        i_dias_previos:  b.i_dias_previos,
        num_rens_proy:   b.num_rens_proy,
        num_rens_otros:  b.num_rens_otros,
        fin_tarea:       b.fin_tarea,
        pag_ancho_total: b.pag_ancho_total,
        largo_concepto:  b.largo_concepto,
    }
}

fn cfg_json(c: &Configura) -> Value {
    json!({
        "nom_empresa":     c.nom_empresa,
        "tipo_unidad":     c.tipo_unidad,
        "image_path":      c.image_path,
        "num_rens_ppto":   c.num_rens_ppto,
        "i_top":           c.i_top,
        "i_rig":           c.i_rig,
        "i_bot":           c.i_bot,
        "i_lef":           c.i_lef,
        "ppto_color_edit": c.ppto_color_edit,
        "color_nivel1":    c.color_nivel1,
        "color_nivel2":    c.color_nivel2,
        "color_nivel3":    c.color_nivel3,
        "color_nivel4":    c.color_nivel4,
        "i_dias_previos":  c.i_dias_previos,
        "num_rens_proy":   c.num_rens_proy,
        "num_rens_otros":  c.num_rens_otros,
        "fin_tarea":       c.fin_tarea,
        "pag_ancho_total": c.pag_ancho_total,
        "largo_concepto":  c.largo_concepto,
    })
}

// ── Cambia configuración ──────────────────────────────────────────────────────

#[utoipa::path(
    put,
    path = "/sistema/configura",
    request_body = ConfiguraInput,
    responses(
        (status = 200, description = "Configuración actualizada",            body = Value),
        (status = 400, description = "Actualización cancelada o error",      body = Value),
    ),
    tag = "SistemaConfigura"
)]
pub async fn cambia_configuracion(
    State(state): State<AppState>,
    Json(body): Json<ConfiguraInput>,
) -> (StatusCode, Json<Value>) {
    debug!("PUT /sistema/configura nom_empresa='{}'", body.nom_empresa);

    let cfg = to_model(body);
    let ret = svc::cambia_configuracion(&state.postgres, &cfg).await;

    if ret.afectado > 0 {
        info!("PUT /sistema/configura ← 200 OK");
        (StatusCode::OK, Json(json!({ "codigo": ret.codigo, "afectado": ret.afectado, "mensaje": ret.mensaje })))
    } else {
        error!("PUT /sistema/configura ← 400 codigo={}", ret.codigo);
        (StatusCode::BAD_REQUEST, Json(json!({ "codigo": ret.codigo, "mensaje": ret.mensaje })))
    }
}

// ── Carga configuración ───────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/sistema/configura",
    responses(
        (status = 200, description = "Configuración del sistema", body = Value),
        (status = 404, description = "Sin configuración",         body = Value),
        (status = 500, description = "Error de base de datos",    body = Value),
    ),
    tag = "SistemaConfigura"
)]
pub async fn carga_configuracion(
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    debug!("GET /sistema/configura");

    match svc::carga_configuracion(&state.postgres).await {
        Ok(Some(c)) => {
            info!("GET /sistema/configura ← 200 OK");
            (StatusCode::OK, Json(cfg_json(&c)))
        }
        Ok(None) => {
            info!("GET /sistema/configura ← 404");
            (StatusCode::NOT_FOUND, Json(json!({ "codigo": -41, "mensaje": "Sin configuración registrada" })))
        }
        Err(rc) => {
            error!("GET /sistema/configura ← 500 codigo={}", rc.codigo);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "codigo": rc.codigo, "mensaje": rc.mensaje })))
        }
    }
}
