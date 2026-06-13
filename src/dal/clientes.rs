// Programa...: clientes
// Descripción: Operaciones de la tabla cpa_Clientes (multi-tenant)
// Origen.....: oClientes.cs
//
// Stored Procedures que usa:
//   sp_cpa_ClientesAdd      → alta              (..., tenant_id)
//   sp_cpa_ClientesDel      → baja              (id, tenant_id)
//   sp_cpa_ClientesUpd      → cambios           (id, ..., tenant_id)
//   sp_cpa_ClientesQry      → consulta por id   (id, tenant_id)
//   sp_cpa_ClientesLstAct   → lista activos     (activos, tenant_id)
//   sp_cpa_clientes_lookup  → autocomplete      (q, limit, tenant_id)
//   sp_cpa_clientes_seed    → seed por tenant   (tenant_id)

use crate::domain::models::clients::Clientes;
use crate::domain::models::lookup::LookupItem;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_ClientesAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, cte: &Clientes, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ClientesAdd($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
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
// BAJA — sp_cpa_ClientesDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet.
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_ClientesDel($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
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
pub async fn cambios(pool: &PgPool, cte: &Clientes, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ClientesUpd($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
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
// CONSULTA — sp_cpa_ClientesQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Clientes>, ReturnCode> {
    let result = sqlx::query_as::<_, Clientes>(
        "SELECT * FROM arqeth.sp_cpa_ClientesQry($1, $2)"
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
// NOMBRE CLIENTE — sp_cpa_ClientesQry
// Reutiliza el mismo SP, devuelve solo el nombre vía ReturnCode.mensaje.
// ─────────────────────────────────────────────
pub async fn nombre_cliente(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT nombre FROM arqeth.sp_cpa_ClientesQry($1, $2)"
    )
    .bind(id)
    .bind(tenant_id)
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
pub async fn obtiene_clientes(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<Clientes>, ReturnCode> {
    let result = sqlx::query_as::<_, Clientes>(
        "SELECT * FROM arqeth.sp_cpa_ClientesLstAct($1, $2)"
    )
    .bind(activos)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -30, afectado: 0, mensaje: "No hay clientes".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_clientes_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_cpa_clientes_seed($1)")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
}

// ─────────────────────────────────────────────
// LOOKUP — sp_cpa_clientes_lookup
// Autocomplete: devuelve (id, etiqueta) para alimentar un combobox.
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM arqeth.sp_cpa_clientes_lookup($1, $2, $3)"
    )
    .bind(q)
    .bind(limit)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CLIENTES ACCESIBLES (portal) — clientes de los proyectos que el usuario ve
// según su perfil (nivel 1=todos / 2=su grupo>0 / 3=asignados>0). Runtime query
// (sin SP) para no requerir reaplicar la BD.
// ─────────────────────────────────────────────
pub async fn clientes_accesibles(
    pool: &PgPool,
    tenant_id: Uuid,
    grupo: i32,
    gn_usr_id: i32,
    nivel: i32,
) -> Vec<LookupItem> {
    sqlx::query_as::<_, LookupItem>(
        r#"SELECT DISTINCT c.id, c.nombre AS etiqueta
           FROM arqeth.cpa_proyectos p
           JOIN arqeth.cpa_clientes  c ON c.id = p.cliente
           WHERE p.tenant_id = $1 AND p.activo = TRUE
             AND ( $4 = 1
                   OR ($4 = 2  AND $2 > 0 AND EXISTS (
                         SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                         WHERE a.proyecto_id = p.id AND a.gn_id = $2))
                   OR ($4 >= 3 AND $3 > 0 AND EXISTS (
                         SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                         WHERE a.proyecto_id = p.id AND a.gn_usr_id = $3)) )
           ORDER BY c.nombre"#,
    )
    .bind(tenant_id)
    .bind(grupo)
    .bind(gn_usr_id)
    .bind(nivel)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

// ¿El usuario puede ver el estado de cuenta de este cliente? (nivel 1 = sí).
pub async fn cliente_accesible(
    pool: &PgPool,
    tenant_id: Uuid,
    grupo: i32,
    gn_usr_id: i32,
    nivel: i32,
    cliente: i32,
) -> bool {
    if nivel <= 1 {
        return true;
    }
    sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(
            SELECT 1 FROM arqeth.cpa_proyectos p
            WHERE p.tenant_id = $1 AND p.activo = TRUE AND p.cliente = $5
              AND ( ($4 = 2  AND $2 > 0 AND EXISTS (
                        SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                        WHERE a.proyecto_id = p.id AND a.gn_id = $2))
                    OR ($4 >= 3 AND $3 > 0 AND EXISTS (
                        SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                        WHERE a.proyecto_id = p.id AND a.gn_usr_id = $3)) ) )"#,
    )
    .bind(tenant_id)
    .bind(grupo)
    .bind(gn_usr_id)
    .bind(nivel)
    .bind(cliente)
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

// Autocomplete de clientes VISIBLES (de los proyectos del usuario) + texto `q`.
pub async fn lookup_accesibles(
    pool: &PgPool, tenant_id: Uuid, grupo: i32, gn_usr_id: i32, nivel: i32, q: &str, limit: i32,
) -> Vec<LookupItem> {
    sqlx::query_as::<_, LookupItem>(
        r#"SELECT DISTINCT c.id, c.nombre AS etiqueta
           FROM arqeth.cpa_proyectos p
           JOIN arqeth.cpa_clientes  c ON c.id = p.cliente
           WHERE p.tenant_id = $1 AND p.activo = TRUE
             AND ( $4 = 1
                   OR ($4 = 2  AND $2 > 0 AND EXISTS (
                         SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                         WHERE a.proyecto_id = p.id AND a.gn_id = $2))
                   OR ($4 >= 3 AND $3 > 0 AND EXISTS (
                         SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                         WHERE a.proyecto_id = p.id AND a.gn_usr_id = $3)) )
             AND ($5 = '' OR c.nombre ILIKE '%' || $5 || '%')
           ORDER BY c.nombre
           LIMIT $6"#,
    )
    .bind(tenant_id).bind(grupo).bind(gn_usr_id).bind(nivel).bind(q).bind(limit)
    .fetch_all(pool).await.unwrap_or_default()
}
