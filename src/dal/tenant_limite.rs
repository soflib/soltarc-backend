// Programa...: tenant_limite
// Descripción: tope de proyectos por tenant según el plan comprado.
//   El plan vive en Medusa (auth.tenant_payments, otra DB) y el SP de creación
//   de proyectos corre en arqeth → no puede leerlo cross-DB. Por eso guardamos
//   el tope en arqeth.cpa_tenant_limites en el registro (donde sí tenemos el plan)
//   y la creación de proyectos lo valida local en sp_cpa_proyectosadd.
//
// Stored Procedures que usa:
//   sp_cpa_tenant_limite_set → upsert del tope (tenant_id, max)

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

/// Upsert del tope de proyectos del tenant en arqeth.cpa_tenant_limites.
/// `max = None` se guarda como NULL (ilimitado).
pub async fn set_limite(pool: &PgPool, tenant_id: Uuid, max: Option<i32>) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT arqeth.sp_cpa_tenant_limite_set($1, $2)")
        .bind(tenant_id)
        .bind(max)
        .execute(pool)
        .await?;
    Ok(())
}
