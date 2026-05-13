// Programa...: gn_usuarios
// Descripción: Operaciones de la tabla gnUsuarios
// Origen.....: oGNUsuarios.cs
//
// Stored Procedures que usa:
//   sp_GN_UsuariosAdd    → alta
//   sp_GN_UsuariosDel    → baja
//   sp_GN_UsuariosUpd    → cambios
//   sp_GN_UsuariosQry    → consulta por id
//   sp_GN_UsuariosLstAll → lista todos

use crate::domain::models::gn_usuarios::GnUsuarios;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — sp_GN_UsuariosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, usr: &GnUsuarios) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_UsuariosAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
    )
    .bind(&usr.user_id)
    .bind(usr.grupo_negocio)
    .bind(usr.activo)
    .bind(usr.nivel)
    .bind(usr.opt_cte_1)
    .bind(usr.opt_cte_2)
    .bind(usr.opt_cte_3)
    .bind(usr.opt_cte_4)
    .bind(usr.opt_cte_5)
    .bind(usr.opt_cte_6)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BAJA — sp_GN_UsuariosDel
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_UsuariosDel($1)"
    )
    .bind(id)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: "Baja ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -21, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIOS — sp_GN_UsuariosUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, usr: &GnUsuarios) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_UsuariosUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
    )
    .bind(usr.id)
    .bind(&usr.user_id)
    .bind(usr.grupo_negocio)
    .bind(usr.activo)
    .bind(usr.nivel)
    .bind(usr.opt_cte_1)
    .bind(usr.opt_cte_2)
    .bind(usr.opt_cte_3)
    .bind(usr.opt_cte_4)
    .bind(usr.opt_cte_5)
    .bind(usr.opt_cte_6)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: "Actualización Ok".to_string() },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CONSULTA — sp_GN_UsuariosQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<GnUsuarios>, ReturnCode> {
    let result = sqlx::query_as::<_, GnUsuarios>(
        "SELECT * FROM arqeth.sp_GN_UsuariosQry($1)"
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
// OBTIENE TODO — sp_GN_UsuariosLstAll
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool) -> Result<Vec<GnUsuarios>, ReturnCode> {
    let result = sqlx::query_as::<_, GnUsuarios>(
        "SELECT * FROM arqeth.sp_GN_UsuariosLstAll()"
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -21, afectado: 0, mensaje: "No hay entradas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}
