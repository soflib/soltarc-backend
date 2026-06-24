// Programa...: ppto_seed
// Descripción: Siembra por tenant de la sección Presupuestos (catálogos +
//              presupuesto demo con partidas WBS).
//
// Orquestador SQL: arqeth.sp_ppto_seed(tenant_id) — definido en
// seed_presupuestos.sql. Internamente siembra tipos_costo, unidades,
// costos_estimados y un presupuesto de ejemplo. Idempotente por tenant.
//
// Se invoca desde el alta de admin en api/handlers/auth/identity.rs, igual que
// el resto de los seeds por tenant (catalogos/proveedores/centros/clientes).

use sqlx::PgPool;
use uuid::Uuid;

pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid, lang: &str) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_ppto_seed($1, $2)")
        .bind(tenant_id)
        .bind(lang)
        .fetch_one(pool)
        .await
}
