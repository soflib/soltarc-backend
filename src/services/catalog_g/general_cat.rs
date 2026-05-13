// Programa...: cat_general
// Descripción: Mantenimiento al catálogo general
// Origen.....: CatGeneral.aspx.cs
//
// DAL que usa (ya migrado en catalog_g.rs):
//   alta            → sp_cpa_CatalogoAdd
//   baja            → sp_cpa_CatalogoDel
//   cambios         → sp_cpa_CatalogoUpd
//   consulta        → sp_cpa_CatalogoQry
//   obtiene_todo    → sp_cpa_CatalogoLstAll
//
// Tipos del catálogo general:
//   0 Sistema           4 Tipo proveedor
//   1 Estado proyecto   5 Bancos
//   2 Tipo proyecto     6 Tipo Tarea
//   3 Tipo Persona moral 7 Estado PPTO
//                        8 Estado Partidas

use crate::dal::catalog_g as svc;
use crate::infrastructure::db::return_code::ReturnCode;
use crate::domain::models::catalog_g::{
    CatalogGInput,
    CatalogG,
};
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA
// Devuelve ReturnCode; en caso de éxito retC.afectado
// contiene el Id generado
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cat: &CatalogGInput) -> ReturnCode {
    let dal_cat = CatalogG {
        id:          None,
        tipo:        Some(cat.tipo as i16),
        nombre:      Some(cat.nombre.clone()),
        activo:      Some(cat.activo),
        comentarios: cat.comentarios.clone(),
    };
    svc::alta(pool, &dal_cat).await
}

// ─────────────────────────────────────────────
// BAJA
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    svc::baja(pool, id).await
}

// ─────────────────────────────────────────────
// CAMBIOS
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, cat: &CatalogGInput) -> ReturnCode {
    let dal_cat = CatalogG {
        id:          Some(cat.id.unwrap_or(0)),
        tipo:        Some(cat.tipo as i16),
        nombre:      Some(cat.nombre.clone()),
        activo:      Some(cat.activo),
        comentarios: cat.comentarios.clone(),
    };
    svc::cambios(pool, &dal_cat).await
}

// ─────────────────────────────────────────────
// CONSULTA
// Devuelve Ok(Some(CatalogoG)) si existe,
//          Ok(None)            si no existe,
//          Err(ReturnCode)     en error de BD
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<CatalogG>, ReturnCode> {
    svc::consulta(pool, id).await
}

// ─────────────────────────────────────────────
// OBTIENE TODO
// Reemplaza DSObtieneTodo() + CargaArbol()
// Devuelve la lista completa para que el handler
// construya la respuesta (árbol, JSON, etc.)
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    svc::obtiene_todo(pool).await
}

pub async fn obtiene_por_tipo(pool: &PgPool, tipo: i32) -> Result<Vec<CatalogG>, ReturnCode> {
    svc::obtiene_por_tipo(pool, tipo).await
}

pub async fn obtiene_tipos(pool: &PgPool) -> Result<Vec<CatalogG>, ReturnCode> {
    svc::obtiene_tipos(pool).await
}