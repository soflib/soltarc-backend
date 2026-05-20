// Programa...: proveedores
// Descripción: Operaciones de la tabla cpa_Proveedores
// Origen.....: oProveedores.cs
//
// Stored Procedures que usa:
//   sp_cpa_ProveedoresAdd    → alta
//   sp_cpa_ProveedoresDel    → baja
//   sp_cpa_ProveedoresUpd    → cambio
//   sp_cpa_ProveedoresQry    → consulta por id
//   sp_cpa_ProveedoresLstAll → lista todos (activos o no)

use crate::domain::models::lookup::LookupItem;
use crate::domain::models::proveedores::Proveedores;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_ProveedoresAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, prov: &Proveedores) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ProveedoresAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
    )
    .bind(&prov.nombre)
    .bind(&prov.contacto)
    .bind(&prov.direccion)
    .bind(&prov.telefono)
    .bind(&prov.mail)
    .bind(&prov.cuenta_banco)
    .bind(prov.tipo)
    .bind(prov.giro)
    .bind(&prov.comentarios)
    .bind(prov.activo)
    .bind(&prov.rfc)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_ProveedoresDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, proveedor: i32) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_ProveedoresDel($1)"
    )
    .bind(proveedor)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -23, afectado: 0, mensaje: "Baja Cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIO — sp_cpa_ProveedoresUpd
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, prov: &Proveedores) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ProveedoresUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
    )
    .bind(prov.id.unwrap_or(0))  // id es Option<i32> — 0 nunca debería llegar aquí
    .bind(&prov.nombre)
    .bind(&prov.contacto)
    .bind(&prov.direccion)
    .bind(&prov.telefono)
    .bind(&prov.mail)
    .bind(&prov.cuenta_banco)
    .bind(prov.tipo)
    .bind(prov.giro)
    .bind(&prov.comentarios)
    .bind(prov.activo)
    .bind(&prov.rfc)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_ProveedoresQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<Proveedores>, ReturnCode> {
    let result = sqlx::query_as::<_, Proveedores>(
        "SELECT * FROM arqeth.sp_cpa_ProveedoresQry($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -41, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CARGA PROVEEDORES — sp_cpa_ProveedoresLstAll
// ─────────────────────────────────────────────
pub async fn carga_proveedores(pool: &PgPool, activos: bool) -> Result<Vec<Proveedores>, ReturnCode> {
    let result = sqlx::query_as::<_, Proveedores>(
        "SELECT * FROM arqeth.sp_cpa_ProveedoresLstAll($1)"
    )
    .bind(activos)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -10, afectado: 0, mensaje: "No hay proveedores".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// LOOKUP — sp_cpa_proveedores_lookup
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, limit: i32) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM arqeth.sp_cpa_proveedores_lookup($1, $2)"
    )
    .bind(q)
    .bind(limit)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}
