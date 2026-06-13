// Programa...: presupuesto
// Descripción: Operaciones de la tabla ppto_Presupuesto
// Origen.....: oPresupuesto.cs
//
// Stored Procedures que usa:
//   ppto_sp_Presupuesto_Add    → alta
//   ppto_sp_Presupuesto_DEL    → baja
//   ppto_sp_Presupuesto_UPD    → cambio
//   ppto_sp_Presupuesto_QRY    → consulta por id
//   ppto_sp_Presupuesto_LSTACT → lista activos por grupo/usuario/nivel
//
// Nota: CargaPPTOS() recibía controles UI (DropDownList, ListBox, DataList).
//       Se migra a Vec<Presupuesto>; la capa de presentación construye los controles.
//       Las variables de seguridad (GpoId, UsrId, UsrNivel) ahora son parámetros explícitos.

use crate::domain::models::lookup::LookupItem;
use crate::domain::models::presupuesto::Presupuesto;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — ppto_sp_Presupuesto_Add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, ppto: &Presupuesto, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_Presupuesto_Add($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
    )
    .bind(&ppto.nombre)
    .bind(&ppto.descripcion)
    .bind(&ppto.direccion)
    .bind(&ppto.comentarios)
    .bind(&ppto.fecha)
    .bind(ppto.cliente)
    .bind(ppto.activo)
    .bind(ppto.estado)
    .bind(&ppto.pie_pagina)
    .bind(ppto.gn_id)
    .bind(ppto.gn_user_id)
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
// BAJA — ppto_sp_Presupuesto_DEL
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_Presupuesto_DEL($1, $2)"
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
// CAMBIO — ppto_sp_Presupuesto_UPD
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, ppto: &Presupuesto, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_Presupuesto_UPD($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
    )
    .bind(ppto.id.unwrap_or(0))  // id es Option<i32> — 0 nunca debería llegar aquí
    .bind(&ppto.nombre)
    .bind(&ppto.descripcion)
    .bind(&ppto.direccion)
    .bind(&ppto.comentarios)
    .bind(&ppto.fecha)
    .bind(ppto.cliente)
    .bind(ppto.activo)
    .bind(ppto.estado)
    .bind(&ppto.pie_pagina)
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
// CONSULTA — ppto_sp_Presupuesto_QRY
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Presupuesto>, ReturnCode> {
    let result = sqlx::query_as::<_, Presupuesto>(
        "SELECT * FROM arqeth.ppto_sp_Presupuesto_QRY($1, $2)"
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
// CARGA PRESUPUESTOS — ppto_sp_Presupuesto_LSTACT
// gpo_neg, gpo_user_id y usr_nivel vienen del contexto de seguridad del caller
// ─────────────────────────────────────────────
pub async fn carga_presupuestos(
    pool: &PgPool,
    gpo_neg: i32,
    gpo_user_id: i32,
    usr_nivel: i32,
    activos: bool,
    tenant_id: Uuid,
) -> Result<Vec<Presupuesto>, ReturnCode> {
    let result = sqlx::query_as::<_, Presupuesto>(
        "SELECT * FROM arqeth.ppto_sp_Presupuesto_LSTACT($1, $2, $3, $4, $5)"
    )
    .bind(gpo_neg)
    .bind(gpo_user_id)
    .bind(usr_nivel)
    .bind(activos)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -80, afectado: 0, mensaje: "No hay presupuestos disponibles".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -85, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// LOOKUP — ppto_sp_presupuesto_lookup
// Acepta filtro opcional por cliente (None = todos).
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, cliente: Option<i32>, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM arqeth.ppto_sp_presupuesto_lookup($1, $2, $3, $4)"
    )
    .bind(q)
    .bind(cliente)
    .bind(limit)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -95, afectado: 0, mensaje: e.to_string() }),
    }
}

// Autocomplete de presupuestos VISIBLES para el usuario + texto `q`.
// El acceso DERIVA DEL CLIENTE: el presupuesto es de un cliente (p.cliente), y el
// usuario lo ve si ese cliente está entre los clientes que ya alcanza por sus
// proyectos (según nivel/grupo/asignación). gn_id/gn_user_id del presupuesto no se
// usan (nunca se asignan). Admin/nivel 1 → todos.
pub async fn lookup_accesibles(
    pool: &PgPool, tenant_id: Uuid, grupo: i32, gn_usr_id: i32, nivel: i32, q: &str, limit: i32,
) -> Vec<LookupItem> {
    sqlx::query_as::<_, LookupItem>(
        r#"SELECT p.id, p.nombre AS etiqueta
           FROM arqeth.ppto_presupuesto p
           WHERE p.tenant_id = $1 AND p.activo = TRUE
             AND ( $4 = 1
                   OR p.cliente IN (
                        SELECT DISTINCT pr.cliente FROM arqeth.cpa_proyectos pr
                        WHERE pr.tenant_id = $1 AND pr.activo = TRUE
                          AND ( ($4 = 2  AND $2 > 0 AND EXISTS (
                                    SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                                    WHERE a.proyecto_id = pr.id AND a.gn_id = $2))
                                OR ($4 >= 3 AND $3 > 0 AND EXISTS (
                                    SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                                    WHERE a.proyecto_id = pr.id AND a.gn_usr_id = $3)) ) ) )
             AND ($5 = '' OR p.nombre ILIKE '%' || $5 || '%')
           ORDER BY p.nombre
           LIMIT $6"#,
    )
    .bind(tenant_id).bind(grupo).bind(gn_usr_id).bind(nivel).bind(q).bind(limit)
    .fetch_all(pool).await.unwrap_or_default()
}

// ¿El usuario puede ver este presupuesto? (deriva del cliente; nivel 1 = sí).
pub async fn presupuesto_accesible(
    pool: &PgPool, tenant_id: Uuid, grupo: i32, gn_usr_id: i32, nivel: i32, presupuesto: i32,
) -> bool {
    sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS(
            SELECT 1 FROM arqeth.ppto_presupuesto p
            WHERE p.tenant_id = $1 AND p.id = $5
              AND ( $4 = 1
                    OR p.cliente IN (
                         SELECT DISTINCT pr.cliente FROM arqeth.cpa_proyectos pr
                         WHERE pr.tenant_id = $1 AND pr.activo = TRUE
                           AND ( ($4 = 2  AND $2 > 0 AND EXISTS (
                                     SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                                     WHERE a.proyecto_id = pr.id AND a.gn_id = $2))
                                 OR ($4 >= 3 AND $3 > 0 AND EXISTS (
                                     SELECT 1 FROM arqeth.cpa_proyecto_asignaciones a
                                     WHERE a.proyecto_id = pr.id AND a.gn_usr_id = $3)) ) ) ) )"#,
    )
    .bind(tenant_id).bind(grupo).bind(gn_usr_id).bind(nivel).bind(presupuesto)
    .fetch_one(pool).await.unwrap_or(false)
}
