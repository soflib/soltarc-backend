// Programa...: egresos
// Descripción: Operaciones de la tabla cpa_Egresos
// Origen.....: oEgresos.cs
//
// Stored Procedures que usa:
//   sp_cpa_EgresosAdd           → alta
//   sp_cpa_EgresosDel           → baja
//   sp_cpa_EgresosUpd           → cambios
//   sp_cpa_EgresosQry           → consulta por id
//   sp_cpa_EgresosQryProyxRef   → egresos por proyecto (xref)
//
// Nota: TotalEgresos() usaba SQL inline en C#.
//       Se migra a SP dedicado para mantener consistencia
//       con el resto del DAL.

use crate::domain::models::egresos::{EgresoConTotal, Egresos, EgresosFilter};
use crate::domain::models::lookup::PageOf;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_EgresosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, egr: &Egresos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_EgresosAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"
    )
    .bind(egr.fecha)          // $1  p_fecha
    .bind(egr.banco)          // $2  p_banco
    .bind(&egr.cuenta)        // $3  p_cuenta
    .bind(&egr.forma_pago)    // $4  p_forma_pago
    .bind(egr.centro_costo)   // $5  p_centro_costo
    .bind(egr.monto)          // $6  p_monto
    .bind(&egr.referencia)    // $7  p_referencia
    .bind(&egr.comentario)    // $8  p_comentario
    .bind(egr.fecha_aplica)   // $9  p_fecha_aplica
    .bind(egr.proyecto)       // $10 p_proyecto
    .bind(egr.proveedor)      // $11 p_proveedor
    .bind(egr.usuario)        // $12 p_usuario
    .bind(tenant_id)          // $13 p_tenant_id
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_EgresosDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, egreso: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_EgresosDel($1, $2)"
    )
    .bind(egreso)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_cpa_EgresosUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, egr: &Egresos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_EgresosUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)"
    )
    .bind(egr.id.unwrap_or(0))  // $1  p_id
    .bind(egr.fecha)            // $2  p_fecha
    .bind(egr.banco)            // $3  p_banco
    .bind(&egr.cuenta)          // $4  p_cuenta
    .bind(&egr.forma_pago)      // $5  p_forma_pago
    .bind(egr.centro_costo)     // $6  p_centro_costo
    .bind(egr.monto)            // $7  p_monto
    .bind(&egr.referencia)      // $8  p_referencia
    .bind(&egr.comentario)      // $9  p_comentario
    .bind(egr.fecha_aplica)     // $10 p_fecha_aplica
    .bind(egr.proyecto)         // $11 p_proyecto
    .bind(egr.proveedor)        // $12 p_proveedor
    .bind(egr.usuario)          // $13 p_usuario
    .bind(tenant_id)            // $14 p_tenant_id
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_EgresosQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Egresos>, ReturnCode> {
    let result = sqlx::query_as::<_, Egresos>(
        "SELECT * FROM arqeth.sp_cpa_EgresosQry($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CARGA EGRESOS PROYECTO XREF — sp_cpa_EgresosQryProyxRef
// ─────────────────────────────────────────────
pub async fn carga_egresos_proy_xref(pool: &PgPool, proyecto: i32, tenant_id: Uuid) -> Result<Vec<Egresos>, ReturnCode> {
    let result = sqlx::query_as::<_, Egresos>(
        "SELECT * FROM arqeth.sp_cpa_EgresosQryProyxRef($1, $2)"
    )
    .bind(proyecto)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    // Lista vacía es respuesta válida (proyecto sin egresos), no error.
    // El handler la convierte en 200 con egresos: [].
    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -95, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// TOTAL EGRESOS — sp_cpa_EgresosTotalProy
// El SQL inline original se migra a SP dedicado.
// El SP debe implementar:
//   SELECT COALESCE(SUM(egr_Monto), 0) FROM cpa_Egresos WHERE egr_Proyecto = $1
// ─────────────────────────────────────────────
pub async fn total_egresos(pool: &PgPool, proyecto: i32, tenant_id: Uuid) -> Result<rust_decimal::Decimal, ReturnCode> {
    let result = sqlx::query_scalar::<_, rust_decimal::Decimal>(
        "SELECT arqeth.sp_cpa_EgresosTotalProy($1, $2)"
    )
    .bind(proyecto)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(total) => Ok(total),
        Err(e)    => Err(ReturnCode { codigo: -105, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEARCH — sp_cpa_egresos_search
// Filtros (proyecto/proveedor/centro_costo/fechas) + ILIKE en
// referencia/comentario/proveedor/proyecto + paginación.
// El SP devuelve total_count en cada fila.
// ─────────────────────────────────────────────
pub async fn search(pool: &PgPool, f: &EgresosFilter, tenant_id: Uuid) -> Result<PageOf<Egresos>, ReturnCode> {
    let rows = sqlx::query_as::<_, EgresoConTotal>(
        "SELECT * FROM arqeth.sp_cpa_egresos_search($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(f.proyecto)
    .bind(f.proveedor)
    .bind(f.centro_costo)
    .bind(f.fecha_ini)
    .bind(f.fecha_fin)
    .bind(f.q.as_deref())
    .bind(f.offset)
    .bind(f.limit)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match rows {
        Ok(rows) => {
            let total = rows.first().map(|r| r.total_count).unwrap_or(0);
            let size  = if f.limit > 0 { f.limit } else { 25 };
            let page  = if size > 0 { (f.offset / size) + 1 } else { 1 };
            let items = rows.into_iter().map(|r| r.egreso).collect();
            Ok(PageOf { items, total, page, size })
        }
        Err(e) => Err(ReturnCode { codigo: -115, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_egresos_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid, usuario: Uuid) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_cpa_egresos_seed($1, $2)")
        .bind(tenant_id)
        .bind(usuario)
        .fetch_one(pool)
        .await
}
