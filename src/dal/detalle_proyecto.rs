// Programa...: detalle_proyecto
// Descripción: Operaciones de la tabla cpa_DetalleProyectos
// Origen.....: oDetalleProyecto.cs
//
// Stored Procedures que usa:
//   sp_cpa_DetalleProyectosAdd        → alta
//   sp_cpa_DetalleProyectosDel        → baja
//   sp_cpa_DetalleProyectosUpd        → cambios
//   sp_cpa_DetalleProyectosQry        → consulta por id
//   sp_cpa_DetalleProyectosQryTareas  → partidas del proyecto
//   sp_cpa_DetalleProyectosQryPry     → tareas del proyecto
//   sp_cpa_Proy_Partidas              → partidas xref
//   pdo_sp_PartidasFecDep_QRYUPDdes   → nodos descendientes
//   pdo_sp_PartidasFecDep_UPDdesFech  → actualiza fechas por nodo
//   sp_cpa_DetProyCopyQry             → consulta diferencias entre proyectos
//   sp_cpa_DetProyCopyUPD             → copia contenido de partidas
//   sp_cpa_DetProyADDQry              → consulta partidas faltantes
//   sp_cpa_DetProyADDFaltantes        → adiciona partidas faltantes

use crate::domain::models::detalle_proyectos::DetalleProyectos;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

// ─────────────────────────────────────────────
// ALTA — sp_cpa_DetalleProyectosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, det: &DetalleProyectos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_DetalleProyectosAdd($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(det.proyecto)
    .bind(det.tipo)
    .bind(det.secuencia)
    .bind(&det.descripcion)
    .bind(&det.comentarios)
    .bind(det.presupuesto)
    .bind(det.fecha_inicio)
    .bind(det.fecha_fin)
    .bind(det.estado)
    .bind(&det.nodo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_cpa_DetalleProyectosDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id_tarea: i32) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_cpa_DetalleProyectosDel($1)"
    )
    .bind(id_tarea)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(rc)) => rc,
        Ok(None)     => ReturnCode { codigo: -21, afectado: 0, mensaje: "-21: Baja cancelada".to_string() },
        Err(e)       => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_cpa_DetalleProyectosUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, det: &DetalleProyectos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_DetalleProyectosUpd($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(det.id)
    .bind(det.proyecto)
    .bind(det.tipo)
    .bind(det.secuencia)
    .bind(&det.descripcion)
    .bind(&det.comentarios)
    .bind(det.presupuesto)
    .bind(det.fecha_inicio)
    .bind(det.fecha_fin)
    .bind(det.estado)
    .bind(&det.nodo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_cpa_DetalleProyectosQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.sp_cpa_DetalleProyectosQry($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// PARTIDAS PROYECTO — sp_cpa_DetalleProyectosQryTareas
// ─────────────────────────────────────────────
pub async fn partidas_proyecto(pool: &PgPool, proyecto: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.sp_cpa_DetalleProyectosQryTareas($1)"
    )
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -50, afectado: 0, mensaje: "No hay tareas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CARGA TAREAS — sp_cpa_DetalleProyectosQryPry
// ─────────────────────────────────────────────
pub async fn carga_tareas(pool: &PgPool, proyecto: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.sp_cpa_DetalleProyectosQryPry($1)"
    )
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -60, afectado: 0, mensaje: "No hay tareas para el proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// CONSULTA PARTIDAS XREF — sp_cpa_Proy_Partidas
// ─────────────────────────────────────────────
pub async fn consulta_partidas_xref(pool: &PgPool, proyecto: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.sp_cpa_Proy_Partidas($1)"
    )
    .bind(proyecto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -81, afectado: 0, mensaje: "No hay datos para las partidas del proyecto".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -85, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// NODOS DESC — pdo_sp_PartidasFecDep_QRYUPDdes
// ─────────────────────────────────────────────
pub async fn nodos_desc(pool: &PgPool, nodo_raiz: &str) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.pdo_sp_PartidasFecDep_QRYUPDdes($1)"
    )
    .bind(nodo_raiz)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay partidas en el Nodo".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ACTUALIZA FECHAS — pdo_sp_PartidasFecDep_UPDdesFech
// ─────────────────────────────────────────────
pub async fn actualiza_fechas(
    pool: &PgPool,
    nodo: &str,
    fecha_ini: Date,
    fecha_fin: Date,
    estado: i32,
    fecha_termino: Date,
) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.pdo_sp_PartidasFecDep_UPDdesFech($1, $2, $3, $4, $5)"
    )
    .bind(nodo)
    .bind(fecha_ini)
    .bind(fecha_fin)
    .bind(estado)
    .bind(fecha_termino)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 10,  afectado: n, mensaje: "Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -11, afectado: 0, mensaje: "Error de proceso, reintente de nuevo".to_string() },
        Err(e)         => ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// COPIA PARTIDAS QRY — sp_cpa_DetProyCopyQry
// Consulta diferencias entre proyecto origen y destino
// ─────────────────────────────────────────────
pub async fn copia_cont_partidas_qry(pool: &PgPool, pry_org: i32, pry_des: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.sp_cpa_DetProyCopyQry($1, $2)"
    )
    .bind(pry_org)
    .bind(pry_des)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -11, afectado: 0, mensaje: "No hay partidas en origen o destino para copiar".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// COPIA CONTENIDO PARTIDAS — sp_cpa_DetProyCopyUPD
// Actualiza partidas con diferencias del origen al destino
// ─────────────────────────────────────────────
pub async fn copia_contenido_partidas(pool: &PgPool, origen: i32, destino: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_DetProyCopyUPD($1, $2)"
    )
    .bind(origen)
    .bind(destino)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: "Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -21, afectado: 0, mensaje: "No hay partidas en origen o destino para copiar".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// ADICIONA PARTIDAS QRY — sp_cpa_DetProyADDQry
// Muestra partidas del origen que no están en el destino
// ─────────────────────────────────────────────
pub async fn adiciona_partidas_qry(pool: &PgPool, pry_org: i32, pry_des: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let result = sqlx::query_as::<_, DetalleProyectos>(
        "SELECT * FROM arqeth.sp_cpa_DetProyADDQry($1, $2)"
    )
    .bind(pry_org)
    .bind(pry_des)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -31, afectado: 0, mensaje: "No hay partidas en origen o destino para adicionar".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ADICIONA PARTIDAS FALTANTES — sp_cpa_DetProyADDFaltantes
// Adiciona al destino las partidas que solo existen en el origen
// ─────────────────────────────────────────────
pub async fn adiciona_partidas_faltantes(pool: &PgPool, origen: i32, destino: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_cpa_DetProyADDFaltantes($1, $2)"
    )
    .bind(origen)
    .bind(destino)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 40,  afectado: n, mensaje: "Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -41, afectado: 0, mensaje: "No hay partidas en origen o destino para agregar".to_string() },
        Err(e)         => ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() },
    }
}
