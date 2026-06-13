// Programa...: gn_usuarios
// Descripción: Operaciones de la tabla gnUsuarios (multi-tenant)
// Origen.....: oGNUsuarios.cs
//
// Stored Procedures que usa (todos con p_tenant_id):
//   sp_GN_UsuariosAdd    → alta              (..., tenant_id)
//   sp_GN_UsuariosDel    → baja              (id, tenant_id)
//   sp_GN_UsuariosUpd    → cambios           (id, ..., tenant_id)
//   sp_GN_UsuariosQry    → consulta por id   (id, tenant_id)
//   sp_GN_UsuariosLstAll → lista todos       (tenant_id)

use crate::domain::models::gn_usuarios::GnUsuarios;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use uuid::Uuid;

// ─────────────────────────────────────────────
// ALTA — sp_GN_UsuariosAdd
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, usr: &GnUsuarios, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_UsuariosAdd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"
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
// BAJA — sp_GN_UsuariosDel
// El SP devuelve (codigo, mensaje, afectado) como ResultSet.
// ─────────────────────────────────────────────
pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_as::<_, ReturnCode>(
        "SELECT codigo, mensaje, afectado FROM arqeth.sp_GN_UsuariosDel($1, $2)"
    )
    .bind(id)
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
// CAMBIOS — sp_GN_UsuariosUpd
// ─────────────────────────────────────────────
pub async fn cambios(pool: &PgPool, usr: &GnUsuarios, tenant_id: Uuid) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.sp_GN_UsuariosUpd($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)"
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
// CONSULTA — sp_GN_UsuariosQry
// ─────────────────────────────────────────────
pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<GnUsuarios>, ReturnCode> {
    let result = sqlx::query_as::<_, GnUsuarios>(
        "SELECT * FROM arqeth.sp_GN_UsuariosQry($1, $2)"
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
// OBTIENE TODO — sp_GN_UsuariosLstAll
// ─────────────────────────────────────────────
pub async fn obtiene_todo(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<GnUsuarios>, ReturnCode> {
    let result = sqlx::query_as::<_, GnUsuarios>(
        "SELECT * FROM arqeth.sp_GN_UsuariosLstAll($1)"
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -21, afectado: 0, mensaje: "No hay entradas".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// LINK — sp_gn_usuarios_link: liga el perfil de negocio al usuario real (UUID).
// ─────────────────────────────────────────────
pub async fn link(pool: &PgPool, tenant_id: Uuid, usuario_uuid: Uuid, email: &str, nivel: i32) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT arqeth.sp_gn_usuarios_link($1, $2, $3, $4)")
        .bind(tenant_id)
        .bind(usuario_uuid)
        .bind(email)
        .bind(nivel)
        .fetch_one(pool)
        .await
}

// ─────────────────────────────────────────────
// PERFIL — sp_gn_usuarios_perfil: (grupo, gn_usr_id, nivel) del usuario logueado.
// Sin perfil → (0, 0, 1): nivel 1 (ve todo), para no bloquear usuarios sin perfil.
// ─────────────────────────────────────────────
pub async fn perfil_de(pool: &PgPool, tenant_id: Uuid, usuario_uuid: Uuid) -> (i32, i32, i32) {
    let row = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT grupo_negocio, gn_usr_id, nivel FROM arqeth.sp_gn_usuarios_perfil($1, $2)"
    )
    .bind(tenant_id)
    .bind(usuario_uuid)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    row.unwrap_or((0, 0, 1))
}

// Perfil a partir de los datos del JWT: Admin → (0,0,1) = ve todo; el resto por
// su gn_usuarios; user_id inválido o sin perfil → (0,0,1). Reusable en handlers.
pub async fn perfil_de_auth(pool: &PgPool, tenant_id: Uuid, user_id: &str, role: &str) -> (i32, i32, i32) {
    if role.eq_ignore_ascii_case("Admin") {
        return (0, 0, 1);
    }
    match Uuid::parse_str(user_id) {
        Ok(uid) => perfil_de(pool, tenant_id, uid).await,
        Err(_)  => (0, 0, 1),
    }
}

// ─────────────────────────────────────────────
// SYNC — liga cada usuario real (uuid, email, role) a su perfil gn_usuarios.
// Default de nivel por rol: admin/finanzas=1 (todo), arquitecto=2 (grupo), resto=3.
// Devuelve cuántos se procesaron. Idempotente.
// ─────────────────────────────────────────────
pub async fn sync_for_tenant(pool: &PgPool, tenant_id: Uuid, usuarios: &[(Uuid, String, String)]) -> i64 {
    let mut n = 0i64;
    for (uuid, email, role) in usuarios {
        let nivel = match role.to_lowercase().as_str() {
            "admin" | "finanzas" => 1,
            "arquitecto"         => 2,
            _                    => 3,
        };
        if link(pool, tenant_id, *uuid, email, nivel).await.is_ok() {
            n += 1;
        }
    }
    n
}
