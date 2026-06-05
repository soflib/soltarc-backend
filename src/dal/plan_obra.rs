// Programa...: plan_obra
// Descripción: Procesos del plan de la obra
// Origen.....: oPlanObra.cs
//
// Stored Procedures:
//   pdo_sp_PartidasFecDep_UPD      → actualiza fechas/estado de una partida
//   pdo_sp_PartidasFecDep_QRY      → consulta partidas del proyecto
//   sp_cpa_Proy_AvancePlan         → obtiene avance del plan
//   pdo_sp_PartidasFecDep_QRYExist → verifica cuántas partidas tienen fechas creadas
//   pdo_sp_PartidasFecDep_ADDFyD   → crea el plan inicial de fechas desde el PPTO
//   pdo_sp_PartidasFecDep_QRYUPDdes→ descendientes de un nodo para réplica de fechas

use crate::domain::models::plan_obra::PlanObra;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub struct PlanStatus {
    pub total_partidas: i32,
    pub con_fecha:      i32,
}

#[derive(sqlx::FromRow)]
struct PlanStatusRow {
    total_partidas: Option<i32>,
    con_fecha:      Option<i32>,
}

// ─────────────────────────────────────────────
// PARTIDA UPD FECHA — pdo_sp_PartidasFecDep_UPD
// ─────────────────────────────────────────────
pub async fn partida_upd_fecha(pool: &PgPool, pla: &PlanObra) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.pdo_sp_PartidasFecDep_UPD($1, $2, $3, $4, $5)"
    )
    .bind(pla.id)
    .bind(pla.fecha_ini)
    .bind(pla.fecha_fin)
    .bind(pla.estado)
    .bind(pla.fecha_termina)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 10,  afectado: n, mensaje: "Actualizó ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -11, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// PARTIDA PROYECTO — pdo_sp_PartidasFecDep_QRY
// ─────────────────────────────────────────────
pub async fn partida_proyecto(pool: &PgPool, proyecto: i32) -> Result<Vec<PlanObra>, ReturnCode> {
    let result = sqlx::query_as::<_, PlanObra>(
        "SELECT * FROM arqeth.pdo_sp_PartidasFecDep_QRY($1)"
    )
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay partidas en el proyecto, se requiere al menos (1)".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// OBTIENE AVANCE — sp_cpa_Proy_AvancePlan
// ─────────────────────────────────────────────
pub async fn obtiene_avance(pool: &PgPool, proyecto: i32, nivel: i32) -> Result<Vec<PlanObra>, ReturnCode> {
    let result = sqlx::query_as::<_, PlanObra>(
        "SELECT * FROM arqeth.sp_cpa_Proy_AvancePlan($1, $2)"
    )
    .bind(proyecto)
    .bind(nivel)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -10, afectado: 0, mensaje: "No hay partidas en el proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EXISTE PLAN — pdo_sp_PartidasFecDep_QRYExist
// Verifica cuántas partidas del proyecto tienen
// fechas ya creadas vs. cuántas partidas existen.
// CreaFechayDepende.aspx lo usa para decidir si
// habilitar el botón "Crear".
// ─────────────────────────────────────────────
pub async fn existe_plan(pool: &PgPool, proyecto: i32) -> Result<PlanStatus, ReturnCode> {
    let result = sqlx::query_as::<_, PlanStatusRow>(
        "SELECT total_partidas, con_fecha FROM arqeth.pdo_sp_PartidasFecDep_QRYExist($1)"
    )
    .bind(proyecto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => match (row.total_partidas, row.con_fecha) {
            (Some(t), Some(c)) => Ok(PlanStatus { total_partidas: t, con_fecha: c }),
            _ => Err(ReturnCode { codigo: -17, afectado: 0, mensaje: "Sin datos del proyecto".to_string() }),
        },
        Ok(None) => Err(ReturnCode { codigo: -10, afectado: 0, mensaje: "Proyecto no encontrado".to_string() }),
        Err(e)   => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CREA PLAN — pdo_sp_PartidasFecDep_ADDFyD
// Crea los registros de fecha/dependencia para
// las partidas que aún no los tienen.
// Returns: número de registros creados (> 0 = ok)
// ─────────────────────────────────────────────
pub async fn crea_plan(
    pool: &PgPool,
    proyecto: i32,
    fecha_ini: Date,
    fecha_fin: Date,
    estado: i32,
) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.pdo_sp_PartidasFecDep_ADDFyD($1, $2, $3, $4)"
    )
    .bind(proyecto)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .bind(estado)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 10,  afectado: n, mensaje: format!("Se crearon {} registros de fecha y dependencia", n) },
        Ok(_)          => ReturnCode { codigo: -11, afectado: 0, mensaje: "Error al crear el plan, o no hay partidas pendientes".to_string() },
        Err(e)         => ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// DESCENDIENTES NODO — pdo_sp_PartidasFecDep_QRYUPDdes
// Devuelve las partidas descendientes de un nodo.
// ReplicaFechas.aspx lo usa para previsualizar
// qué partidas hija serán afectadas antes de
// llamar a ActualizaFechas (ya en Part B DAL).
// ─────────────────────────────────────────────
pub async fn descendientes_nodo(pool: &PgPool, proyecto: i32, nodo: &str) -> Result<Vec<PlanObra>, ReturnCode> {
    // La SP devuelve la forma completa de cpa_detalle_proyectos (fecha_inicio, etc.);
    // seleccionamos y aliasamos las columnas que espera PlanObra.
    // p_proyecto acota a un solo proyecto (los nodos se repiten entre proyectos).
    let result = sqlx::query_as::<_, PlanObra>(
        "SELECT id, fecha_inicio AS fecha_ini, fecha_fin, estado, comentarios, fecha_termina, nodo, descripcion \
         FROM arqeth.pdo_sp_PartidasFecDep_QRYUPDdes($1, $2)"
    )
    .bind(proyecto)
    .bind(nodo)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -10, afectado: 0, mensaje: "No hay partidas descendientes en el nodo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}
