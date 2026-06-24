// Programa...: tenant_limite
// Descripción: topes por tenant según el plan comprado (proyectos + almacenamiento).
//   El plan vive en Medusa (auth.tenant_payments, otra DB) y los SP de arqeth
//   (creación de proyectos, alta de archivos) no pueden leerlo cross-DB. Por eso
//   guardamos los topes en arqeth.cpa_tenant_limites en el registro (donde sí
//   tenemos el plan): la creación de proyectos los valida local en
//   sp_cpa_proyectosadd y la cuota de almacenamiento en sp_cpa_archivo_add.
//
// Stored Procedures que usa:
//   sp_cpa_tenant_limite_set → upsert de los topes (tenant_id, max_proyectos, max_storage_bytes)

use sqlx::PgPool;
use uuid::Uuid;

/// Máximo de proyectos según el plan comprado. None = ilimitado
/// (plan `dedicated`, plan desconocido, o registro admin sin plan).
pub fn max_for_plan(plan: &str) -> Option<i32> {
    match plan {
        "plan5"  => Some(5),
        "plan10" => Some(10),
        "plan20" => Some(20),
        _        => None,
    }
}

/// Cuota de almacenamiento (bytes) según el plan comprado: plan5→10GB,
/// plan10→15GB, plan20→20GB. None = sin cuota propia (plan `dedicated`,
/// desconocido, o admin sin plan) → el SP usa su default (25GB).
pub fn storage_for_plan(plan: &str) -> Option<i64> {
    const GB: i64 = 1024 * 1024 * 1024;
    match plan {
        "plan5"  => Some(10 * GB),
        "plan10" => Some(15 * GB),
        "plan20" => Some(20 * GB),
        _        => None,
    }
}

/// Upsert de los topes del tenant en arqeth.cpa_tenant_limites.
/// `None` se guarda como NULL (proyectos ilimitados / cuota por default).
/// `idioma`:
///   Some("es"|"en") → fija el idioma de los seeds del tenant (alta de admin).
///   None            → no toca el idioma (p.ej. upgrade de plan: solo cambia topes).
pub async fn set_limite(
    pool: &PgPool,
    tenant_id: Uuid,
    max_proyectos: Option<i32>,
    max_storage_bytes: Option<i64>,
    idioma: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT arqeth.sp_cpa_tenant_limite_set($1, $2, $3, $4)")
        .bind(tenant_id)
        .bind(max_proyectos)
        .bind(max_storage_bytes)
        .bind(idioma)
        .execute(pool)
        .await?;
    Ok(())
}
