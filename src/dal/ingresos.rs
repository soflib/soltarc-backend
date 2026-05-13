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

use crate::domain::models::ingresos::Ingresos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_IngresosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, ing: &Ingresos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_IngresosAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
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
pub async fn baja(pool: &PgPool, ingreso: i32) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_IngresosDel($1)"
    )
    .bind(ingreso)
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
pub async fn cambios(pool: &PgPool, ing: &Ingresos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_IngresosUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
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
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Ingresos>, ReturnCode> {
    let result = sqlx::query_as::<_, Ingresos>(
        "SELECT * FROM arqeth.sp_cpa_IngresosQry($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}
