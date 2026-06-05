// Programa...: services::operaciones::xref
// Descripción: Lógica de negocio para xref detalle proyecto ↔ finanzas
// Origen.....: oXref_DetalleProy_Finan.cs
//
// DAL que usa:
//   crate::dal::xref_detalle_proy_finan::*

use crate::dal::xref_detalle_proy_finan as dal;
use crate::domain::models::xref_detalle_proy_finan::{XrefDetalleProyFinan, XrefSaldo};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

pub async fn alta(pool: &PgPool, xref: &XrefDetalleProyFinan) -> ReturnCode {
    dal::alta(pool, xref).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambio(pool: &PgPool, xref: &XrefDetalleProyFinan) -> ReturnCode {
    dal::cambio(pool, xref).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<XrefDetalleProyFinan>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn egresos_a_partidas(pool: &PgPool, partida: i32) -> Result<Vec<XrefDetalleProyFinan>, ReturnCode> {
    dal::egresos_a_partidas(pool, partida).await
}

pub async fn egresos_no_asignados(pool: &PgPool, proyecto: i32) -> Result<Vec<XrefDetalleProyFinan>, ReturnCode> {
    dal::egresos_no_asignados(pool, proyecto).await
}

pub async fn saldo(pool: &PgPool, transaccion: i32) -> Result<XrefSaldo, ReturnCode> {
    dal::saldo(pool, transaccion).await
}
