// Programa...: cat_general (multi-tenant)
// Descripción: Mantenimiento al catálogo general
// Origen.....: CatGeneral.aspx.cs
//
// DAL que usa (ya migrado en catalog_g.rs, todas las funciones reciben tenant_id):
//   alta            → sp_cpa_CatalogoAdd
//   baja            → sp_cpa_CatalogoDel
//   cambios         → sp_cpa_CatalogoUpd
//   consulta        → sp_cpa_CatalogoQry
//   obtiene_todo    → sp_cpa_CatalogoLstAll
//
// Tipos del catálogo general:
//   1 Estado proyecto    5 Bancos
//   2 Tipo proyecto      6 Tipo Tarea
//   3 Tipo Persona moral 7 Estado PPTO
//   4 Tipo proveedor     8 Estado Partidas
//
// El tenant_id lo recibe cada función del service (lo aporta el handler
// desde AuthUser, extraído del JWT).

use crate::dal::catalog_g as svc;
use crate::infrastructure::db::return_code::ReturnCode;
use crate::domain::models::catalog_g::{
    CatalogGInput,
    CatalogG,
};
use crate::domain::models::lookup::LookupItem;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA
// Devuelve ReturnCode; en caso de éxito ret.afectado
// contiene el Id generado.
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cat: &CatalogGInput, tenant_id: Uuid) -> ReturnCode {
    let dal_cat = CatalogG {
        id:          None,
        tipo:        Some(cat.tipo as i16),
        nombre:      Some(cat.nombre.clone()),
        activo:      Some(cat.activo),
        comentarios: cat.comentarios.clone(),
        tenant_id:   Some(tenant_id),
    };
    svc::alta(pool, &dal_cat, tenant_id).await
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    svc::baja(pool, id, tenant_id).await
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cat: &CatalogGInput, tenant_id: Uuid) -> ReturnCode {
    let dal_cat = CatalogG {
        id:          Some(cat.id.unwrap_or(0)),
        tipo:        Some(cat.tipo as i16),
        nombre:      Some(cat.nombre.clone()),
        activo:      Some(cat.activo),
        comentarios: cat.comentarios.clone(),
        tenant_id:   Some(tenant_id),
    };
    svc::cambios(pool, &dal_cat, tenant_id).await
}

// ─────────────────────────────────────────────
// CONSULTA
// Devuelve Ok(Some(CatalogoG)) si existe y es visible al tenant,
//          Ok(None)            si no existe / no visible,
//          Err(ReturnCode)     en error de BD.
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<CatalogG>, ReturnCode> {
    svc::consulta(pool, id, tenant_id).await
}

// ─────────────────────────────────────────────
// OBTIENE TODO
// Devuelve la lista visible al tenant (globales + propios).
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    svc::obtiene_todo(pool, tenant_id).await
}

pub async fn obtiene_por_tipo(pool: &PgPool, tipo: i32, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    svc::obtiene_por_tipo(pool, tipo, tenant_id).await
}

pub async fn obtiene_tipos(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<CatalogG>, ReturnCode> {
    svc::obtiene_tipos(pool, tenant_id).await
}

// ─────────────────────────────────────────────
// LOOKUP — autocomplete catálogo por tipo
// Ej: tipo=5 → bancos, tipo=3 → tipos persona moral
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, tipo: i16, q: &str, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    svc::lookup(pool, tipo, q, limit, tenant_id).await
}
