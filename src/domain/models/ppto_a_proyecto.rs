// Programa...: models/ppto_a_proyecto.rs
// Descripción: Modelo para la transferencia de presupuesto a proyecto
// Origen.....: oPPTOaProyecto.cs — ppto_sp_cpa_CreaPartidas_De_PPTO_QRY

use rust_decimal::Decimal;
use utoipa::ToSchema;

// NodoPartida
// Origen: ppto_sp_cpa_CreaPartidas_De_PPTO_QRY(@ppto, @proyecto)
//
// Propósito: Representa un nodo/partida del presupuesto que será copiado
//            al proyecto. Se usa para previsualizar qué partidas se crearán
//            antes de ejecutar la transferencia.
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct NodoPartida {
    pub id: Option<i32>,
    pub nodo: Option<String>,
    pub concepto: Option<String>,
    pub nivel: Option<i32>,
    pub unidad: Option<i32>,
    #[schema(value_type = Option<f64>)]
    pub cantidad: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub precio_u: Option<Decimal>,
    pub calculo: Option<i32>,
    pub presupuesto: Option<i32>,
}
