// Programa...: ingresos
// Descripción: Operaciones de la tabla cpa_Ingresos
// Origen.....: oIngresos.cs
//
// Stored Procedures que usa:
//   sp_cpa_IngresosAdd  → alta
//   sp_cpa_IngresosDel  → baja
//   sp_cpa_IngresosUpd  → cambios
//   sp_cpa_IngresosQry  → consulta por id
//
// Nota: @Usuario solo se envía en Alta (igual que en C#)

use crate::domain::models::ingresos::{IngresoConTotal, Ingresos, IngresosFilter};
use crate::domain::models::lookup::PageOf;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_IngresosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, ing: &Ingresos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_IngresosAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
    )
    .bind(ing.fecha)          // $1  p_fecha
    .bind(ing.banco)          // $2  p_banco
    .bind(&ing.cuenta)        // $3  p_cuenta
    .bind(&ing.forma_pago)    // $4  p_forma_pago
    .bind(ing.proyecto)       // $5  p_proyecto
    .bind(ing.monto)          // $6  p_monto
    .bind(&ing.referencia)    // $7  p_referencia
    .bind(&ing.comentario)    // $8  p_comentario
    .bind(ing.fecha_aplica)   // $9  p_fecha_aplica
    .bind(ing.cliente)        // $10 p_cliente
    .bind(ing.usuario_ms)     // $11 p_usuario_ms
    .bind(tenant_id)          // $12 p_tenant_id
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_IngresosDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, ingreso: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_IngresosDel($1, $2)"
    )
    .bind(ingreso)
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
// CAMBIOS — sp_cpa_IngresosUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, ing: &Ingresos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_IngresosUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)"
    )
    .bind(ing.id.unwrap_or(0))  // $1  p_id
    .bind(ing.fecha)            // $2  p_fecha
    .bind(ing.banco)            // $3  p_banco
    .bind(&ing.cuenta)          // $4  p_cuenta
    .bind(&ing.forma_pago)      // $5  p_forma_pago
    .bind(ing.proyecto)         // $6  p_proyecto
    .bind(ing.monto)            // $7  p_monto
    .bind(&ing.referencia)      // $8  p_referencia
    .bind(&ing.comentario)      // $9  p_comentario
    .bind(ing.fecha_aplica)     // $10 p_fecha_aplica
    .bind(ing.cliente)          // $11 p_cliente
    .bind(ing.usuario_ms)       // $12 p_usuario_ms
    .bind(tenant_id)            // $13 p_tenant_id
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_IngresosQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Ingresos>, ReturnCode> {
    let result = sqlx::query_as::<_, Ingresos>(
        "SELECT * FROM arqeth.sp_cpa_IngresosQry($1, $2)"
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
// LISTA — sp_cpa_ingresos_lstact
// Filtros opcionales por proyecto y/o cliente (None = sin filtro).
// ─────────────────────────────────────────────
pub async fn lista(pool: &PgPool, proyecto: Option<i32>, cliente: Option<i32>, tenant_id: Uuid) -> Result<Vec<Ingresos>, ReturnCode> {
    let result = sqlx::query_as::<_, Ingresos>(
        "SELECT * FROM arqeth.sp_cpa_ingresos_lstact($1, $2, $3)"
    )
    .bind(proyecto)
    .bind(cliente)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEARCH — sp_cpa_ingresos_search
// Filtros + ILIKE en referencia/comentario/cliente/proyecto + paginación.
// El SP devuelve total_count en cada fila para evitar un segundo viaje.
// ─────────────────────────────────────────────
pub async fn search(pool: &PgPool, f: &IngresosFilter, tenant_id: Uuid) -> Result<PageOf<Ingresos>, ReturnCode> {
    let rows = sqlx::query_as::<_, IngresoConTotal>(
        "SELECT * FROM arqeth.sp_cpa_ingresos_search($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(f.proyecto)
    .bind(f.cliente)
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
            let items = rows.into_iter().map(|r| r.ingreso).collect();
            Ok(PageOf { items, total, page, size })
        }
        Err(e) => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_ingresos_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid, usuario: Uuid, lang: &str) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_cpa_ingresos_seed($1, $2, $3)")
        .bind(tenant_id)
        .bind(usuario)
        .bind(lang)
        .fetch_one(pool)
        .await
}
