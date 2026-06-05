// Programa...: proyectos
// Descripción: Operaciones de la tabla cpa_Proyectos (multi-tenant)
// Origen.....: oProyectos.cs
//
// Stored Procedures que usa:
//   sp_cpa_ProyectosAdd       → alta              (..., tenant_id)
//   sp_cpa_ProyectosDel       → baja              (id, tenant_id)
//   sp_cpa_ProyectosUpd       → cambios           (id, ..., tenant_id)
//   sp_cpa_ProyectosUpdGpoUsr → actualiza grupo/usuario del proyecto
//   sp_cpa_ProyectosQry       → consulta por id   (id, tenant_id)
//   sp_cpa_ProyectosLstAct    → lista activos     (activos, tenant_id)
//   sp_cpa_ProyectosTotalPPTO → total presupuesto del proyecto
//   sp_cpa_proyectos_lookup   → autocomplete      (q, limit, tenant_id)
//   sp_cpa_proyectos_seed     → seed por tenant   (tenant_id)
//
// SQL inline migrado a SPs dedicados:
//   sp_cpa_ProyectoClienteNombre → reemplaza JOIN inline para nombre cliente
//   sp_cpa_ProyectoDirImagenes   → reemplaza SELECT inline para directorio

use crate::domain::models::lookup::LookupItem;
use crate::domain::models::proyectos::Proyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_ProyectosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, proy: &Proyectos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ProyectosAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)"
    )
    .bind(proy.tipo)
    .bind(&proy.nombre)
    .bind(&proy.descripcion)
    .bind(&proy.direccion)
    .bind(&proy.comentarios)
    .bind(proy.estado)
    .bind(proy.presupuesto)
    .bind(proy.fecha_ini)
    .bind(proy.fecha_fin)
    .bind(&proy.asignado)
    .bind(proy.cliente)
    .bind(proy.activo)
    .bind(&proy.dir_imagenes)
    .bind(proy.gn_id)
    .bind(proy.gn_usr_id)
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
// BAJA — sp_cpa_ProyectosDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, proyecto: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_ProyectosDel($1, $2)"
    )
    .bind(proyecto)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIO — sp_cpa_ProyectosUpd
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, proy: &Proyectos, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ProyectosUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)"
    )
    .bind(proy.id)
    .bind(proy.tipo)
    .bind(&proy.nombre)
    .bind(&proy.descripcion)
    .bind(&proy.direccion)
    .bind(&proy.comentarios)
    .bind(proy.estado)
    .bind(proy.presupuesto)
    .bind(proy.fecha_ini)
    .bind(proy.fecha_fin)
    .bind(&proy.asignado)
    .bind(proy.cliente)
    .bind(proy.activo)
    .bind(&proy.dir_imagenes)
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
// GPO USR PROYECTO — sp_cpa_ProyectosUpdGpoUsr
// ─────────────────────────────────────────────
pub async fn gpo_usr_proyecto(pool: &PgPool, proyecto: i32, grupo: i32, usuario: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_ProyectosUpdGpoUsr($1, $2, $3)"
    )
    .bind(proyecto)
    .bind(grupo)
    .bind(usuario)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_ProyectosQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Proyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, Proyectos>(
        "SELECT * FROM arqeth.sp_cpa_ProyectosQry($1, $2)"
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
// LLENA PROYECTOS — sp_cpa_ProyectosLstAct
// ─────────────────────────────────────────────
pub async fn llena_proyectos(pool: &PgPool, activos: bool, tenant_id: Uuid) -> Result<Vec<Proyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, Proyectos>(
        "SELECT * FROM arqeth.sp_cpa_ProyectosLstAct($1, $2)"
    )
    .bind(activos)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -60, afectado: 0, mensaje: "No hay proyectos activos".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CLIENTE PROYECTO — sp_cpa_ProyectoClienteNombre
// El JOIN inline se migra a SP dedicado.
// El SP debe implementar:
//   SELECT cte_Nombre FROM cpa_Proyectos
//   JOIN cpa_Clientes ON cte_Id = pry_Cliente
//   WHERE pry_Id = $1
// ─────────────────────────────────────────────
pub async fn cliente_proyecto(pool: &PgPool, proyecto: i32) -> Result<String, ReturnCode> {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT arqeth.sp_cpa_ProyectoClienteNombre($1)"
    )
    .bind(proyecto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(nombre)) => Ok(nombre),
        Ok(None)         => Err(ReturnCode { codigo: -75, afectado: 0, mensaje: "No se encontró el cliente".to_string() }),
        Err(e)           => Err(ReturnCode { codigo: -75, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// DIR PROYECTO — sp_cpa_ProyectoDirImagenes
// El SELECT inline se migra a SP dedicado.
// El SP debe implementar:
//   SELECT pry_DirImagenes FROM cpa_Proyectos WHERE pry_Id = $1
// ─────────────────────────────────────────────
pub async fn dir_proyecto(pool: &PgPool, proyecto: i32) -> Result<String, ReturnCode> {
    if proyecto == 0 {
        return Err(ReturnCode { codigo: -73, afectado: 0, mensaje: "El proyecto es inválido".to_string() });
    }

    let result = sqlx::query_scalar::<_, Option<String>>(
        "SELECT arqeth.sp_cpa_ProyectoDirImagenes($1)"
    )
    .bind(proyecto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(Some(dir))) if !dir.is_empty() => Ok(dir),
        Ok(Some(Some(_)))                      => Err(ReturnCode { codigo: -71, afectado: 0, mensaje: "No hay directorio asignado".to_string() }),
        Ok(_)                                  => Err(ReturnCode { codigo: -72, afectado: 0, mensaje: "No existe el directorio asignado".to_string() }),
        Err(e)                                 => Err(ReturnCode { codigo: -76, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// TOTAL PPTO — sp_cpa_ProyectosTotalPPTO
// ─────────────────────────────────────────────
pub async fn total_ppto(pool: &PgPool, proyecto: i32) -> Result<rust_decimal::Decimal, ReturnCode> {
    if proyecto == 0 {
        return Err(ReturnCode { codigo: -81, afectado: 0, mensaje: "El proyecto es inválido".to_string() });
    }

    let result = sqlx::query_scalar::<_, rust_decimal::Decimal>(
        "SELECT arqeth.sp_cpa_ProyectosTotalPPTO($1)"
    )
    .bind(proyecto)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(total)) => Ok(total),
        Ok(None)        => Err(ReturnCode { codigo: -86, afectado: 0, mensaje: "Sin resultado".to_string() }),
        Err(e)          => Err(ReturnCode { codigo: -86, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// LOOKUP — sp_cpa_proyectos_lookup
// Etiqueta: "<nombre del proyecto> — <cliente>"
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, cliente: Option<i32>, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    let result = sqlx::query_as::<_, LookupItem>(
        "SELECT id, etiqueta FROM arqeth.sp_cpa_proyectos_lookup($1, $2, $3, $4)"
    )
    .bind(q)
    .bind(cliente)   // None / 0 = todos los proyectos del tenant
    .bind(limit)
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -95, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// SEED por tenant — sp_cpa_proyectos_seed
// Idempotente: re-llamarlo para el mismo tenant no duplica filas.
// ─────────────────────────────────────────────
pub async fn seed_for_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_cpa_proyectos_seed($1)")
        .bind(tenant_id)
        .fetch_one(pool)
        .await
}
