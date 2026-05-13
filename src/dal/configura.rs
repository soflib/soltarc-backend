// Programa...: configura
// Descripción: Operaciones de configuración del sistema
// Origen.....: oConfigura.cs
//
// Stored Procedures que usa:
//   sp_sys_ConfiguracionUpd       → cambia configuración
//   sp_sys_ConfiguracionQry       → carga configuración
//   sp_sys_AccesosRapidosQRY      → consulta acceso rápido por id
//   sp_sys_AccesosRapidosLSTAll   → lista todos los accesos rápidos
//   sp_sys_AccesosRapidosUPD      → actualiza acceso rápido

use crate::domain::models::accesos_rapidos::AccesosRapidos;
use crate::domain::models::configura::Configura;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// CAMBIA CONFIGURACIÓN — sp_sys_ConfiguracionUpd
// ─────────────────────────────────────────────
pub async fn cambia_configuracion(pool: &PgPool, cfg: &Configura) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_sys_ConfiguracionUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19)"
    )
    .bind(&cfg.nom_empresa)      // $1
    .bind(&cfg.tipo_unidad)      // $2
    .bind(cfg.num_rens_ppto)     // $3
    .bind(&cfg.image_path)       // $4
    .bind(cfg.i_top)             // $5
    .bind(cfg.i_rig)             // $6
    .bind(cfg.i_bot)             // $7
    .bind(cfg.i_lef)             // $8
    .bind(&cfg.ppto_color_edit)  // $9
    .bind(&cfg.color_nivel1)     // $10
    .bind(&cfg.color_nivel2)     // $11
    .bind(&cfg.color_nivel3)     // $12
    .bind(&cfg.color_nivel4)     // $13
    .bind(cfg.i_dias_previos)    // $14
    .bind(cfg.num_rens_proy)     // $15
    .bind(cfg.num_rens_otros)    // $16
    .bind(cfg.fin_tarea)         // $17
    .bind(cfg.pag_ancho_total)   // $18
    .bind(cfg.largo_concepto)    // $19 — faltaba en versión anterior
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización Cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CARGA CONFIGURACIÓN — sp_sys_ConfiguracionQry
// ─────────────────────────────────────────────
pub async fn carga_configuracion(pool: &PgPool) -> Result<Option<Configura>, ReturnCode> {
    let result = sqlx::query_as::<_, Configura>(
        "SELECT * FROM arqeth.sp_sys_ConfiguracionQry()"
    )
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ACCESOS RÁPIDOS QRY — sp_sys_AccesosRapidosQRY
// ─────────────────────────────────────────────
pub async fn accesos_rapidos_qry(pool: &PgPool, id: i32) -> Result<Option<AccesosRapidos>, ReturnCode> {
    let result = sqlx::query_as::<_, AccesosRapidos>(
        "SELECT * FROM arqeth.sp_sys_AccesosRapidosQRY($1)"
    )
    .bind(id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(registro) => Ok(registro),
        Err(e)       => Err(ReturnCode { codigo: -55, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ACCESOS RÁPIDOS LST ALL — sp_sys_AccesosRapidosLSTAll
// ─────────────────────────────────────────────
pub async fn accesos_rapidos_lst_all(pool: &PgPool) -> Result<Vec<AccesosRapidos>, ReturnCode> {
    let result = sqlx::query_as::<_, AccesosRapidos>(
        "SELECT * FROM arqeth.sp_sys_AccesosRapidosLSTAll()"
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -61, afectado: 0, mensaje: "No hay entradas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -65, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ACCESOS RÁPIDOS UPD — sp_sys_AccesosRapidosUPD
// ─────────────────────────────────────────────
pub async fn accesos_rapidos_upd(pool: &PgPool, ar: &AccesosRapidos) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_sys_AccesosRapidosUPD($1, $2, $3, $4)"
    )
    .bind(ar.id)
    .bind(&ar.funcion)
    .bind(&ar.tool_tip)
    .bind(&ar.imagen)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 10,  afectado: n, mensaje: "Cambios realizados".to_string() },
        Ok(_)          => ReturnCode { codigo: -16, afectado: 0, mensaje: "Error en cambios".to_string() },
        Err(e)         => ReturnCode { codigo: -15, afectado: 0, mensaje: e.to_string() },
    }
}