// Programa...: PlanObra
// Descripción: Plan de obra
// Origen.....: ePlanObra.cs

use chrono::NaiveDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, sqlx::FromRow)]
pub struct PlanObra {
    pub id: i32,
    pub fecha_ini: NaiveDateTime,
    pub fecha_fin: NaiveDateTime,
    pub estado: i32,
    pub comentarios: String,
    pub fecha_termina: NaiveDateTime,
    // Ruta jerárquica (ej. "/5/1/1.1/"). Permite resolver descendientes.
    // default: la SP de avance no la devuelve y una DB aún sin migrar tampoco.
    #[sqlx(default)]
    pub nodo: String,
    // Nombre de la partida. Solo lo trae la consulta de descendientes; las
    // demás (qry/avance) devuelven default "" porque ya muestran comentarios.
    #[sqlx(default)]
    pub descripcion: String,
}
