// Programa...: unidades
// Descripción: Operaciones de la tabla ppto_Unidades
// Origen.....: oUnidades.cs
//
// Stored Procedures que usa:
//   ppto_sp_Unidades_Add    → alta
//   ppto_sp_Unidades_DEL    → baja
//   ppto_sp_Unidades_UPD    → cambio
//   ppto_sp_Unidades_QRY    → consulta por id
//   ppto_sp_Unidades_LSTACT → lista activos (ObtieneUnidades y CargaArbolUni)
//
// Nota: ObtieneUnidades() y CargaArbolUni() consultaban la configuración
//       para obtener TiposUnidad y construir etiquetas como "Tipo - Nombre".
//       En Rust ese enriquecimiento se delega al caller o a la capa de servicio;
//       obtiene_unidades() devuelve los datos crudos Vec<Unidades>.
//       Si se requiere el label compuesto, el caller llama a
//       configura::carga_configuracion() y ensambla el texto.

use crate::domain::models::unidades::Unidades;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — ppto_sp_Unidades_Add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, uni: &Unidades, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_Unidades_Add($1, $2, $3, $4, $5)"
    )
    .bind(uni.tipo)
    .bind(&uni.descripcion)
    .bind(&uni.nombre_corto)
    .bind(uni.activa)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — ppto_sp_Unidades_DEL
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_Unidades_DEL($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIO — ppto_sp_Unidades_UPD
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, uni: &Unidades, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_Unidades_UPD($1, $2, $3, $4, $5, $6)"
    )
    .bind(uni.id.unwrap_or(0))  // id es Option<i32> — 0 nunca debería llegar aquí
    .bind(uni.tipo)
    .bind(&uni.descripcion)
    .bind(&uni.nombre_corto)
    .bind(uni.activa)
    .bind(tenant_id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — ppto_sp_Unidades_QRY
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Unidades>, ReturnCode> {
    let result = sqlx::query_as::<_, Unidades>(
        "SELECT * FROM arqeth.ppto_sp_Unidades_QRY($1, $2)"
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
// OBTIENE UNIDADES — ppto_sp_Unidades_LSTACT
// Devuelve datos crudos Vec<Unidades>.
// Si se necesita el label "TipoUnidad - NombreCorto", el caller
// debe obtener la configuración y ensamblar el texto.
// ─────────────────────────────────────────────
pub async fn obtiene_unidades(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<Unidades>, ReturnCode> {
    let result = sqlx::query_as::<_, Unidades>(
        "SELECT * FROM arqeth.ppto_sp_Unidades_LSTACT($1)"
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay Unidades".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}
