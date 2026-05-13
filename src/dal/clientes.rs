// Programa...: clientes
// Descripción: Operaciones de la tabla cpa_Clientes
// Origen.....: oClientes.cs
//
// Stored Procedures que usa:
//   sp_cpa_ClientesAdd    → alta
//   sp_cpa_ClientesDel    → baja
//   sp_cpa_ClientesUpd    → cambios
//   sp_cpa_ClientesQry    → consulta por id / nombre por id
//   sp_cpa_ClientesLstAct → lista activos / todos

use crate::domain::models::clients::Clientes;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_ClientesAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cte: &Clientes) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ClientesAdd($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&cte.nombre)
    .bind(&cte.direccion)
    .bind(&cte.telefono)
    .bind(&cte.mail)
    .bind(&cte.cuenta_banco)
    .bind(&cte.comentarios)
    .bind(cte.tipo)
    .bind(cte.activo)
    .bind(&cte.condiciones)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_ClientesDel
// El SP original devuelve (codigo, mensaje, afectado) como ResultSet
// Se mapea directamente al ReturnCode
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_ClientesDel($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -23, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_cpa_ClientesUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cte: &Clientes) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ClientesUpd($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(cte.id)
    .bind(&cte.nombre)
    .bind(&cte.direccion)
    .bind(&cte.telefono)
    .bind(&cte.mail)
    .bind(&cte.cuenta_banco)
    .bind(&cte.comentarios)
    .bind(cte.tipo)
    .bind(cte.activo)
    .bind(&cte.condiciones)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_ClientesQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Clientes>, ReturnCode> {
    let result = sqlx::query_as::<_, Clientes>(
        "SELECT * FROM arqeth.sp_cpa_ClientesQry($1)"
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
// NOMBRE CLIENTE — sp_cpa_ClientesQry
// Reutiliza el mismo SP, devuelve solo el nombre vía ReturnCode.mensaje
// ─────────────────────────────────────────────
pub async fn nombre_cliente(pool: &PgPool, id: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT nombre FROM arqeth.sp_cpa_ClientesQry($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(nombre)) => ReturnCode { codigo: 0,   afectado: id, mensaje: nombre },
        Ok(None)         => ReturnCode { codigo: -10, afectado: 0,  mensaje: "No hay clientes".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// OBTIENE CLIENTES — sp_cpa_ClientesLstAct
// ─────────────────────────────────────────────
pub async fn obtiene_clientes(pool: &PgPool, activos: bool) -> Result<Vec<Clientes>, ReturnCode> {
    let result = sqlx::query_as::<_, Clientes>(
        "SELECT * FROM arqeth.sp_cpa_ClientesLstAct($1)"
    )
    .bind(activos)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -30, afectado: 0, mensaje: "No hay clientes".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() }),
    }
}
