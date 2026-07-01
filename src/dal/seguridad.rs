// Programa...: seguridad
// Descripción: Carga las variables de seguridad del sistema
// Origen.....: oSeguridad.cs

use crate::domain::models::seguridad::Seguridad;
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct GrupoRow {
    nombre:      Option<String>,
    descripcion: Option<String>,
    activo:      Option<bool>,
}

// ─────────────────────────────────────────────
// CARGA VARIABLES
// ─────────────────────────────────────────────
pub async fn carga_variables(
    pool: &PgPool,
    gpo_id: i32,
    usr_id: i32,
    usr_nivel: i32,
    usr_activo: bool,
) -> Result<Seguridad, ReturnCode> {
    let mut seg = Seguridad {
        gpo_id,
        gpo_nombre:  String::new(),
        gpo_descrip: String::new(),
        gpo_activo:  false,
        usr_id,
        usr_activo,
        usr_nivel,
    };

    consulta_grupo_neg(pool, &mut seg).await?;

    Ok(seg)
}

// ─────────────────────────────────────────────
// CONSULTA GRUPO NEG — sp_GN_GruposQry (privado)
// ─────────────────────────────────────────────
async fn consulta_grupo_neg(pool: &PgPool, seg: &mut Seguridad) -> Result<(), ReturnCode> {
    let result = sqlx::query_as::<_, GrupoRow>(
        "SELECT nombre, descripcion, activo FROM soltarc.sp_GN_GruposQry($1)"
    )
    .bind(seg.gpo_id)
    .fetch_optional(pool)
    .await;

    match result {
        Ok(Some(row)) => {
            seg.gpo_nombre  = row.nombre.unwrap_or_default();
            seg.gpo_descrip = row.descripcion.unwrap_or_default();
            seg.gpo_activo  = row.activo.unwrap_or(false);
            Ok(())
        }
        Ok(None) => {
            seg.gpo_activo = false;
            Ok(())
        }
        Err(e) => Err(ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() }),
    }
}
