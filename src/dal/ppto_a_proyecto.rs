// Programa...: ppto_a_proyecto
// Descripción: Operaciones de transferencia de PPTO a Proyectos
// Origen.....: oPPTOaProyecto.cs
//
// Stored Procedures que usa:
//   ppto_sp_cpa_CreaPartidas_De_PPTO_QRY → consulta nodos/partidas a crear
//   ppto_sp_cpa_CreaPartidas_De_PPTO     → crea partidas en el proyecto
//
// SQL inline migrado a SPs dedicados:
//   sp_cpa_ProyectoConteoPartidas        → reemplaza COUNT(*) inline
//   sp_cpa_ProyectoTipo                  → reemplaza SELECT pry_Tipo inline

use crate::domain::models::ppto_a_proyecto::NodoPartida;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

// ─────────────────────────────────────────────
// CONSULTA NÚMERO DE PARTIDAS — sp_cpa_ProyectoConteoPartidas
// El SQL inline original se migra a SP dedicado.
// El SP debe implementar:
//   SELECT COUNT(*) FROM cpa_DetalleProyectos WHERE prd_Proyecto = $1
// ─────────────────────────────────────────────
pub async fn consulta_numero_partidas(pool: &PgPool, proyecto: i32) -> ReturnCode {
    // El SP es RETURNS TABLE(codigo, mensaje, afectado): hay que leer columnas,
    // no decodificar como escalar. `afectado` trae el COUNT.
    let result = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT codigo, mensaje, afectado FROM soltarc.sp_cpa_ProyectoConteoPartidas($1)"
    )
    .bind(proyecto)
    .fetch_one(pool)
    .await;

    match result {
        Ok((_, _, n)) if n > 0 => ReturnCode { codigo: 10,  afectado: n, mensaje: "El proyecto tiene registros".to_string() },
        Ok((_, _, _))          => ReturnCode { codigo: 11,  afectado: 0, mensaje: "El proyecto está vacío proceda".to_string() },
        Err(e)                 => ReturnCode { codigo: -15, afectado: -1, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CARGA NODOS — ppto_sp_cpa_CreaPartidas_De_PPTO_QRY
// ─────────────────────────────────────────────
pub async fn carga_nodos(pool: &PgPool, ppto: i32, proyecto: i32) -> Result<Vec<NodoPartida>, ReturnCode> {
    let result = sqlx::query_as::<_, NodoPartida>(
        "SELECT * FROM soltarc.ppto_sp_cpa_CreaPartidas_De_PPTO_QRY($1, $2)"
    )
    .bind(ppto)
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        // Lista vacía NO es error (puede que ya estén todas aplicadas al proyecto).
        // Solo un error real de BD es Err → el handler lo expone como 500, no 404.
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CREA PARTIDAS PROYECTO — ppto_sp_cpa_CreaPartidas_De_PPTO
// ─────────────────────────────────────────────
pub async fn crea_partidas_proyecto(
    pool: &PgPool,
    ppto: i32,
    proyecto: i32,
    fecha_ini: Date,
    fecha_fin: Date,
    estado: i32,
    tipo: i32,
) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.ppto_sp_cpa_CreaPartidas_De_PPTO($1,$2,$3,$4,$5,$6)"
    )
    .bind(ppto)
    .bind(proyecto)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .bind(estado)
    .bind(tipo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: format!("Se adicionaron: {} Partidas.", n) },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Error al dar de alta, parámetros inválidos".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// OBTIENE TIPO PROYECTO — sp_cpa_ProyectoTipo
// El SQL inline original se migra a SP dedicado.
// El SP debe implementar:
//   SELECT pry_Tipo FROM cpa_Proyectos WHERE pry_Id = $1
// ─────────────────────────────────────────────
pub async fn obtiene_tipo_proyecto(pool: &PgPool, proyecto: i32) -> Result<i32, ReturnCode> {
    // El SP es RETURNS TABLE(codigo, mensaje, afectado): se leen columnas, no
    // un escalar. codigo=40 → encontrado; `afectado` trae el tipo (puede ser 0,
    // p.ej. "Residencial", que es un tipo válido — NO rechazar tipo=0).
    let result = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT codigo, mensaje, afectado FROM soltarc.sp_cpa_ProyectoTipo($1)"
    )
    .bind(proyecto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some((40, _, tipo))) => Ok(tipo),
        Ok(_)  => Err(ReturnCode { codigo: -41, afectado: 0, mensaje: "Proyecto no encontrado".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}
