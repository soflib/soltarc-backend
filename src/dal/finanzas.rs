// Programa...: finanzas
// Descripción: Procesos financieros
// Origen.....: oFinanzas.cs
//
// Stored Procedures que usa:
//   sp_cpa_FinanzasProyQry          → transacciones financieras por proyecto
//   sp_cpa_FinanzasProySum          → resumen proyectos por grupo/usuario/nivel
//   pdo_sp_Finanzas_DistEgresoProp  → distribuye egreso proporcional por nodo
//   sp_cpa_EgresosProveedorProyecto → egresos por proveedor y proyecto (rango fechas)
//   sp_cpa_IngresosQryGeneral       → ingresos detalle (rango fechas)
//
// Nota: Los métodos que devolvían DataSet en C# ahora devuelven
//       Vec<T> tipado. Se usan modelos dedicados por resultado;
//       ajusta los tipos si tus modelos difieren.

use crate::domain::models::finances::{EgresosProveedorProyecto, IngresosDetalle, ResumenProyecto, TrxFinanciera};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

// ─────────────────────────────────────────────
// TRX FINANCIERAS — sp_cpa_FinanzasProyQry
// ─────────────────────────────────────────────
pub async fn trx_financieras(pool: &PgPool, proyecto: i32) -> Result<Vec<TrxFinanciera>, ReturnCode> {
    let result = sqlx::query_as::<_, TrxFinanciera>(
        "SELECT * FROM soltarc.sp_cpa_FinanzasProyQry($1)"
    )
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay partidas para el proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// LLENA DET PROYECTOS — sp_cpa_FinanzasProySum
// nivel tiene default 5 igual que en C#
// El SP filtra por tenant y por asignaciones (cpa_proyecto_asignaciones).
// ─────────────────────────────────────────────
pub async fn llena_det_proyectos(
    pool: &PgPool,
    tenant_id: uuid::Uuid,
    grupo: i32,
    usuario: i32,
    nivel: i32,        // default: 5
) -> Result<Vec<ResumenProyecto>, ReturnCode> {
    let result = sqlx::query_as::<_, ResumenProyecto>(
        "SELECT * FROM soltarc.sp_cpa_FinanzasProySum($1, $2, $3, $4)"
    )
    .bind(tenant_id)
    .bind(grupo)
    .bind(usuario)
    .bind(nivel)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay proyectos para el Usuario/Grupo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// DISTRIBUYE EGRESO — pdo_sp_Finanzas_DistEgresoProp
// El SP devuelve (afectado, ?, mensaje) como ResultSet
// ─────────────────────────────────────────────
pub async fn distribuye_egreso(pool: &PgPool, egreso: i32, nodo: &str) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM soltarc.pdo_sp_Finanzas_DistEgresoProp($1, $2)"
    )
    .bind(egreso)
    .bind(nodo)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) if rc.afectado > 0 => rc,
        Ok(Some(rc))                    => ReturnCode { codigo: -11, afectado: 0, mensaje: rc.mensaje },
        Ok(None)                        => ReturnCode { codigo: -12, afectado: 0, mensaje: "No hay partidas a distribuir".to_string() },
        Err(e)                          => ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// EGRESOS PROVEEDOR PROYECTO — sp_cpa_EgresosProveedorProyecto
// ─────────────────────────────────────────────
pub async fn egresos_proveedor_proyecto(
    pool: &PgPool,
    tipo_rep: bool,
    fecha_ini: Date,
    fecha_fin: Date,
) -> Result<Vec<EgresosProveedorProyecto>, ReturnCode> {
    let result = sqlx::query_as::<_, EgresosProveedorProyecto>(
        "SELECT * FROM soltarc.sp_cpa_EgresosProveedorProyecto($1, $2, $3)"
    )
    .bind(tipo_rep)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay registros".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// INGRESOS DETALLE — sp_cpa_IngresosQryGeneral
// ─────────────────────────────────────────────
pub async fn ingresos_detalle(
    pool: &PgPool,
    fecha_ini: Date,
    fecha_fin: Date,
) -> Result<Vec<IngresosDetalle>, ReturnCode> {
    let result = sqlx::query_as::<_, IngresosDetalle>(
        "SELECT * FROM soltarc.sp_cpa_IngresosQryGeneral($1, $2)"
    )
    .bind(fecha_ini)
    .bind(fecha_fin)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay registros".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}
