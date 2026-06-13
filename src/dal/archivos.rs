// Programa...: archivos
// Descripción: metadata de archivos del tenant en object storage (Contabo).
//   Los bytes viven en el bucket (key = s3_key); aquí solo la metadata para
//   listar sin LIST S3 y calcular la cuota (SUM(bytes) vs max_storage_bytes,
//   default 25GB). Los archivos de soporte (prefijo support/) NO se registran.
//
// Stored Procedures que usa:
//   sp_cpa_archivo_add → alta + valida cuota (RETURN -30 = cuota llena)
//   sp_cpa_archivo_del → baja; regresa s3_key para borrar del bucket

use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

/// 25 GB — cuota default de almacenamiento por tenant.
pub const DEFAULT_MAX_STORAGE_BYTES: i64 = 26_843_545_600;

#[derive(Debug, sqlx::FromRow)]
pub struct Archivo {
    pub id:         i32,
    pub proyecto:   Option<i32>,
    pub s3_key:     String,
    pub nombre:     String,
    pub mime:       String,
    pub bytes:      i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Alta de metadata validando cuota. Ok(id) o Err(-30 cuota llena / -15 BD).
pub async fn alta(
    pool: &PgPool,
    tenant_id: Uuid,
    proyecto: Option<i32>,
    s3_key: &str,
    nombre: &str,
    mime: &str,
    bytes: i64,
    subido_por: &str,
) -> Result<i32, ReturnCode> {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_archivo_add($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind(tenant_id)
    .bind(proyecto)
    .bind(s3_key)
    .bind(nombre)
    .bind(mime)
    .bind(bytes)
    .bind(subido_por)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => Ok(id),
        Ok(-30)          => Err(ReturnCode { codigo: -30, afectado: 0, mensaje: "Espacio de almacenamiento lleno.".to_string() }),
        Ok(_)            => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "Alta de archivo cancelada".to_string() }),
        Err(e)           => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

/// Baja de metadata; regresa el s3_key (para borrar del bucket) o None si no existe.
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<String>, ReturnCode> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT arqeth.sp_cpa_archivo_del($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
    .fetch_one(pool)
    .await
    .map_err(|e| ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() })
}

/// Archivos de un proyecto del tenant (más recientes primero).
pub async fn lista_proyecto(pool: &PgPool, tenant_id: Uuid, proyecto: i32) -> Result<Vec<Archivo>, ReturnCode> {
    sqlx::query_as::<_, Archivo>(
        "SELECT id, proyecto, s3_key, nombre, mime, bytes, created_at
           FROM arqeth.cpa_tenant_archivos
          WHERE tenant_id = $1 AND proyecto = $2
          ORDER BY created_at DESC"
    )
    .bind(tenant_id)
    .bind(proyecto)
    .fetch_all(pool)
    .await
    .map_err(|e| ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() })
}

/// (bytes usados, bytes máximos) del tenant. Máximo NULL/sin fila → 25GB default.
pub async fn uso(pool: &PgPool, tenant_id: Uuid) -> Result<(i64, i64), ReturnCode> {
    let usados = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(SUM(bytes), 0)::BIGINT FROM arqeth.cpa_tenant_archivos WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_one(pool)
    .await
    .map_err(|e| ReturnCode { codigo: -95, afectado: 0, mensaje: e.to_string() })?;

    let max = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT max_storage_bytes FROM arqeth.cpa_tenant_limites WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ReturnCode { codigo: -95, afectado: 0, mensaje: e.to_string() })?
    .flatten()
    .unwrap_or(DEFAULT_MAX_STORAGE_BYTES);

    Ok((usados, max))
}
