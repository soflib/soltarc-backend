// Programa...: xref_detalle_proy_finan
// Descripción: Operaciones de la tabla de referencia cruzada partidas-egresos
// Origen.....: oXref_DetalleProy_Finan.cs
//
// Stored Procedures que usa:
//   sp_cpa_xpfDetProyFinAdd         → alta
//   sp_cpa_xpfDetProyFinDel         → baja
//   sp_cpa_xpfDetProyFinUpd         → cambio
//   sp_cpa_xpfDetProyFinQry         → consulta por id
//   sp_cpa_Proy_Partidas_Egresos    → egresos a partidas
//   sp_cpa_EgresosNoAplicados       → egresos no asignados del proyecto
//
// Nota: EgresosNoAsignados() ligaba a un GridView en C# y no devolvía nada (void).
//       Se migra a Result<Vec<XrefDetalleProyFinan>, ReturnCode> para consistencia.

use crate::domain::models::xref_detalle_proy_finan::{XrefDetalleProyFinan, XrefSaldo};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_xpfDetProyFinAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, xref: &XrefDetalleProyFinan) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_xpfDetProyFinAdd($1, $2, $3, $4, $5, $6)"
    )
    .bind(xref.partida)
    .bind(xref.tipo)
    .bind(xref.transaccion)
    .bind(&xref.comentario)
    .bind(xref.proyecto)
    .bind(xref.monto_aplica)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_xpfDetProyFinDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, x_ref: i32) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM soltarc.sp_cpa_xpfDetProyFinDel($1)"
    )
    .bind(x_ref)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -25, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIO — sp_cpa_xpfDetProyFinUpd
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, xref: &XrefDetalleProyFinan) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT soltarc.sp_cpa_xpfDetProyFinUpd($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(xref.id)
    .bind(xref.partida)
    .bind(xref.tipo)
    .bind(xref.transaccion)
    .bind(&xref.comentario)
    .bind(xref.proyecto)
    .bind(xref.monto_aplica)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_xpfDetProyFinQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<XrefDetalleProyFinan>, ReturnCode> {
    let result = sqlx::query_as::<_, XrefDetalleProyFinan>(
        "SELECT * FROM soltarc.sp_cpa_xpfDetProyFinQry($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EGRESOS A PARTIDAS — sp_cpa_Proy_Partidas_Egresos
// ─────────────────────────────────────────────
pub async fn egresos_a_partidas(pool: &PgPool, partida: i32) -> Result<Vec<XrefDetalleProyFinan>, ReturnCode> {
    let result = sqlx::query_as::<_, XrefDetalleProyFinan>(
        "SELECT * FROM soltarc.sp_cpa_Proy_Partidas_Egresos($1)"
    )
    .bind(partida)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -51, afectado: 0, mensaje: "No hay partidas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SALDO DE EGRESO — sp_cpa_xref_saldo
// Devuelve (monto_egreso, aplicado, disponible) para mostrar al usuario
// cuánto puede asignar antes de exceder el egreso.
// ─────────────────────────────────────────────
pub async fn saldo(pool: &PgPool, transaccion: i32) -> Result<XrefSaldo, ReturnCode> {
    let result = sqlx::query_as::<_, XrefSaldo>(
        "SELECT * FROM soltarc.sp_cpa_xref_saldo($1)"
    )
    .bind(transaccion)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(s)) => Ok(s),
        Ok(None)    => Err(ReturnCode { codigo: -41, afectado: 0, mensaje: "Egreso no encontrado".to_string() }),
        Err(e)      => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EGRESOS NO ASIGNADOS — sp_cpa_EgresosNoAplicados
// En C# era void y llenaba un GridView. Ahora devuelve Vec<T>.
// ─────────────────────────────────────────────
pub async fn egresos_no_asignados(pool: &PgPool, proyecto: i32) -> Result<Vec<XrefDetalleProyFinan>, ReturnCode> {
    let result = sqlx::query_as::<_, XrefDetalleProyFinan>(
        "SELECT * FROM soltarc.sp_cpa_EgresosNoAplicados($1)"
    )
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay datos para las partidas del proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: e.to_string() }),
    }
}
