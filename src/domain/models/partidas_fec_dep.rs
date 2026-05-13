// Programa...: PartidasFecDep
// Descripción: Tabla de fechas y dependencias
// Origen.....: ePartidasFecDep.cs

use chrono::NaiveDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
pub struct PartidasFecDep {
    pub id: i32,
    pub partida: i32,
    pub fecha_ini: NaiveDateTime,
    pub fecha_fin: NaiveDateTime,
    pub depende_de: i32,
    pub estado: i32,
    pub comentarios: String,
}
