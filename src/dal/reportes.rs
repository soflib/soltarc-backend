// Programa...: reportes
// Descripción: Reportes del sistema
// Origen.....: oReportes.cs
//
// Stored Procedures que usa:
//   sp_cpa_CapturaDiaria          → captura diaria (rango fechas)
//   sp_cpa_IngresosReporte        → reporte de ingresos
//   sp_cpa_IngresosQryCliente     → ingresos por cliente
//   sp_cpa_IngresosEstadoCuenta   → estado de cuenta por cliente
//   ppto_sp_Reportes_PPTO_Totales → gran total de un presupuesto
//   ppto_sp_Reportes_PPTO         → partidas de un presupuesto
//   sp_cpa_GpoDetProysArbol       → árbol de tareas del proyecto
//   sp_cpa_Audita_XREF            → auditoría xref de presupuesto
//   sp_cpa_Proy_AvanceDeObraIng   → ingresos de avance de obra
//   sp_cpa_Proy_AvanceDeObraEgr   → egresos de avance de obra
//   sp_cpa_EgresosQryCenCo        → egresos por centro de costo
//   sp_cpa_EgresosQryProveedor    → egresos por proveedor
//   sp_cpa_FinanzasEgresosAcum    → reporte acumulado de egresos
//   sp_cpa_Reporte_Egresos        → reporte general de egresos por banco
//
// Nota: Los métodos que devolvían SqlDataReader en C# (para streaming de
//       reportes impresos) devuelven aquí Vec<T> tipado.
//       TotalesPPTO acumulaba row a row; se simplifica a scalar.

use crate::domain::models::reportes::{
    CapturaDiaria, EgresosCentroCosto, EgresosProveedor, EgresosReporte,
    EstadoCuenta, IngresosCliente, IngresosReporte, PartidasArbol,
    PartidasPptoReporte, RegistroAvance,
};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

// ─────────────────────────────────────────────
// CAPTURA DIARIA — sp_cpa_CapturaDiaria
// ─────────────────────────────────────────────
pub async fn captura_diaria(pool: &PgPool, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<CapturaDiaria>, ReturnCode> {
    let result = sqlx::query_as::<_, CapturaDiaria>(
        "SELECT * FROM soltarc.sp_cpa_CapturaDiaria($1, $2)"
    )
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// INGRESOS REPORTE — sp_cpa_IngresosReporte
// ─────────────────────────────────────────────
pub async fn ingresos_reporte(pool: &PgPool, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<IngresosReporte>, ReturnCode> {
    let result = sqlx::query_as::<_, IngresosReporte>(
        "SELECT * FROM soltarc.sp_cpa_IngresosReporte($1, $2)"
    )
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones de ingresos seleccionadas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// INGRESOS CLIENTE — sp_cpa_IngresosQryCliente
// ─────────────────────────────────────────────
pub async fn ingresos_cliente(pool: &PgPool, id: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<IngresosCliente>, ReturnCode> {
    let result = sqlx::query_as::<_, IngresosCliente>(
        "SELECT * FROM soltarc.sp_cpa_IngresosQryCliente($1, $2, $3)"
    )
    .bind(id)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ESTADO DE CUENTA — sp_cpa_IngresosEstadoCuenta
// ─────────────────────────────────────────────
pub async fn estado_de_cuenta(pool: &PgPool, id: i32) -> Result<Vec<EstadoCuenta>, ReturnCode> {
    let result = sqlx::query_as::<_, EstadoCuenta>(
        "SELECT * FROM soltarc.sp_cpa_IngresosEstadoCuenta($1)"
    )
    .bind(id)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones para el cliente".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// TOTALES PPTO — ppto_sp_Reportes_PPTO_Totales
// El acumulado row-by-row de C# se simplifica: el SP
// ya debe devolver el gran total como scalar.
// ─────────────────────────────────────────────
pub async fn totales_ppto(pool: &PgPool, presupuesto: i32) -> Result<rust_decimal::Decimal, ReturnCode> {
    let result = sqlx::query_scalar::<_, rust_decimal::Decimal>(
        "SELECT soltarc.ppto_sp_Reportes_PPTO_Totales($1)"
    )
    .bind(presupuesto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(total)) => Ok(total),
        Ok(None)        => Ok(rust_decimal::Decimal::ZERO),
        Err(e)          => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CARGA PARTIDAS — ppto_sp_Reportes_PPTO
// ─────────────────────────────────────────────
pub async fn carga_partidas(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasPptoReporte>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidasPptoReporte>(
        "SELECT * FROM soltarc.ppto_sp_Reportes_PPTO($1)"
    )
    .bind(presupuesto)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay partidas en el presupuesto!".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ÁRBOL TAREAS PROYECTO — sp_cpa_GpoDetProysArbol
// ─────────────────────────────────────────────
pub async fn arbol_tareas_proyecto(pool: &PgPool, proyecto: i32) -> Result<Vec<PartidasArbol>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidasArbol>(
        "SELECT * FROM soltarc.sp_cpa_GpoDetProysArbol($1)"
    )
    .bind(proyecto)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay datos para las partidas del proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// AUDITA XREF — sp_cpa_Audita_XREF
// ─────────────────────────────────────────────
pub async fn audita_xref(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasArbol>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidasArbol>(
        "SELECT * FROM soltarc.sp_cpa_Audita_XREF($1)"
    )
    .bind(presupuesto)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay datos para las partidas del proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// INGRESOS (avance de obra) — sp_cpa_Proy_AvanceDeObraIng
// ─────────────────────────────────────────────
pub async fn ingresos(pool: &PgPool, proyecto: i32) -> Result<Vec<RegistroAvance>, ReturnCode> {
    let result = sqlx::query_as::<_, RegistroAvance>(
        "SELECT * FROM soltarc.sp_cpa_Proy_AvanceDeObraIng($1)"
    )
    .bind(proyecto)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -71, afectado: 0, mensaje: "No hay datos de ingresos".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -75, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EGRESOS (avance de obra) — sp_cpa_Proy_AvanceDeObraEgr
// ─────────────────────────────────────────────
pub async fn egresos(pool: &PgPool, proyecto: i32) -> Result<Vec<RegistroAvance>, ReturnCode> {
    let result = sqlx::query_as::<_, RegistroAvance>(
        "SELECT * FROM soltarc.sp_cpa_Proy_AvanceDeObraEgr($1)"
    )
    .bind(proyecto)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -71, afectado: 0, mensaje: "No hay datos de Egresos".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -75, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EGRESOS CENTROS COSTO — sp_cpa_EgresosQryCenCo
// ─────────────────────────────────────────────
pub async fn egresos_centros_costo(pool: &PgPool, id: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosCentroCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, EgresosCentroCosto>(
        "SELECT * FROM soltarc.sp_cpa_EgresosQryCenCo($1, $2, $3)"
    )
    .bind(id)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -31, afectado: 0, mensaje: "No Hay transacciones".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EGRESOS PROVEEDOR — sp_cpa_EgresosQryProveedor
// ─────────────────────────────────────────────
pub async fn egresos_proveedor(pool: &PgPool, id: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosProveedor>, ReturnCode> {
    let result = sqlx::query_as::<_, EgresosProveedor>(
        "SELECT * FROM soltarc.sp_cpa_EgresosQryProveedor($1, $2, $3)"
    )
    .bind(id)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// EGRESOS REPORTE — sp_cpa_FinanzasEgresosAcum
// ─────────────────────────────────────────────
pub async fn egresos_reporte(pool: &PgPool, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosReporte>, ReturnCode> {
    let result = sqlx::query_as::<_, EgresosReporte>(
        "SELECT * FROM soltarc.sp_cpa_FinanzasEgresosAcum($1, $2)"
    )
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones de Egresos en el rango seleccionado".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// REPORTE GENERAL EGRESOS — sp_cpa_Reporte_Egresos
// ─────────────────────────────────────────────
pub async fn reporte_gral_egresos(pool: &PgPool, banco: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosReporte>, ReturnCode> {
    let result = sqlx::query_as::<_, EgresosReporte>(
        "SELECT * FROM soltarc.sp_cpa_Reporte_Egresos($1, $2, $3)"
    )
    .bind(banco)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool).await;

    match result {
        Ok(l) if !l.is_empty() => Ok(l),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No Hay transacciones de Egresos en el banco / fechas seleccionadas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}
