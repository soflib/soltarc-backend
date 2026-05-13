// Programa...: tipos_costo
// Descripción: Operaciones de la tabla ppto_TiposCosto
// Origen.....: oTiposCosto.cs
//
// Stored Procedures que usa:
//   ppto_sp_TiposCosto_Add    → alta
//   ppto_sp_TiposCosto_DEL    → baja
//   ppto_sp_TiposCosto_UPD    → cambio
//   ppto_sp_TiposCosto_QRY    → consulta por id
//   ppto_sp_TiposCosto_LSTACT → lista activos/todos
//
// Nota: CargaTipos() ligaba a DropDownList/ListBox en C#.
//       Se migra a Vec<TiposCosto>; la capa de presentación decide el binding.
//
// Nota: El código original tenía retC.Codigo = 10 en Cambio() para el caso Ok
//       (debería ser 30). Se corrige a 30 para consistencia con el resto del DAL.

use crate::domain::models::tipos_costo::TiposCosto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — ppto_sp_TiposCosto_Add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, tpo: &TiposCosto) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_TiposCosto_Add($1, $2, $3, $4)"
    )
    .bind(&tpo.nombre)
    .bind(&tpo.descripcion)
    .bind(tpo.activo)
    .bind(&tpo.imagen)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — ppto_sp_TiposCosto_DEL
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_TiposCosto_DEL($1)"
    )
    .bind(id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIO — ppto_sp_TiposCosto_UPD
// Nota: C# original usaba codigo 10 en Ok — corregido a 30
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, tpo: &TiposCosto) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_TiposCosto_UPD($1, $2, $3, $4, $5)"
    )
    .bind(tpo.id.unwrap_or(0))  // id es Option<i32> — 0 nunca debería llegar aquí
    .bind(&tpo.nombre)
    .bind(&tpo.descripcion)
    .bind(tpo.activo)
    .bind(&tpo.imagen)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — ppto_sp_TiposCosto_QRY
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<TiposCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, TiposCosto>(
        "SELECT * FROM arqeth.ppto_sp_TiposCosto_QRY($1)"
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
// CARGA TIPOS — ppto_sp_TiposCosto_LSTACT
// ─────────────────────────────────────────────
pub async fn carga_tipos(pool: &PgPool, activos: bool) -> Result<Vec<TiposCosto>, ReturnCode> {
    let result = sqlx::query_as::<_, TiposCosto>(
        "SELECT * FROM arqeth.ppto_sp_TiposCosto_LSTACT($1)"
    )
    .bind(activos)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: "No hay tipos de costo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}
