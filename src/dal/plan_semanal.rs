// Programa...: plan_semanal
// Descripción: Operaciones del plan semanal
// Origen.....: oPlanSemanal.cs
//
// Stored Procedures que usa:
//   sp_cpa_PlanSemFechas   → obtiene fechas ini/fin y num semanas del proyecto
//   sp_cpa_PlanSemPartidas → carga partidas del plan semanal

use crate::domain::models::plan_semanal::PartidasSemanal;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

// Struct de respuesta de fechas del proyecto
pub struct Fechas {
    pub fecha_ini:   Date,
    pub fecha_fin:   Date,
    pub num_semanas: i32,
}

#[derive(sqlx::FromRow)]
struct FechasRow {
    fecha_ini:   Option<Date>,
    fecha_fin:   Option<Date>,
    num_semanas: Option<i32>,
}

// ─────────────────────────────────────────────
// FECHAS — sp_cpa_PlanSemFechas
// ─────────────────────────────────────────────
pub async fn fechas(pool: &PgPool, proyecto: i32) -> Result<Fechas, ReturnCode> {
    let result = sqlx::query_as::<_, FechasRow>(
        "SELECT fecha_ini, fecha_fin, num_semanas FROM arqeth.sp_cpa_PlanSemFechas($1)"
    )
    .bind(proyecto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            match (row.fecha_ini, row.fecha_fin, row.num_semanas) {
                (Some(fi), Some(ff), Some(ns)) => Ok(Fechas {
                    fecha_ini:   fi,
                    fecha_fin:   ff,
                    num_semanas: ns,
                }),
                _ => Err(ReturnCode { codigo: -17, afectado: 0, mensaje: "No hay datos para el proyecto".to_string() }),
            }
        }
        Ok(None) => Err(ReturnCode { codigo: -10, afectado: 0, mensaje: "No hay registros".to_string() }),
        Err(e)   => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CARGA PARTIDAS — sp_cpa_PlanSemPartidas
// ─────────────────────────────────────────────
pub async fn carga_partidas(
    pool: &PgPool,
    proyecto: i32,
    fecha_ini: Date,
    nivel: i32,
) -> Result<Vec<PartidasSemanal>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidasSemanal>(
        "SELECT * FROM arqeth.sp_cpa_PlanSemPartidas($1, $2, $3)"
    )
    .bind(proyecto)
    .bind(fecha_ini)
    .bind(nivel)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -20, afectado: 0, mensaje: "No hay registros".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}
