// Programa...: Configura
// Descripción: Tabla de configuración del sistema
// Origen.....: eConfigura.cs

use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct Configura {
    pub nom_empresa: String,
    pub tipo_unidad: String,
    pub image_path: String,
    pub num_rens_ppto: i32,
    pub i_top: i32,
    pub i_rig: i32,
    pub i_bot: i32,
    pub i_lef: i32,
    pub ppto_color_edit: String,
    pub color_nivel1: String,
    pub color_nivel2: String,
    pub color_nivel3: String,
    pub color_nivel4: String,
    pub i_dias_previos: i32,
    pub num_rens_proy: i32,
    pub num_rens_otros: i32,
    pub fin_tarea: i32,
    pub pag_ancho_total: i32,
    pub largo_concepto: i32,
}
