// Programa...: services::operaciones::proyectos
// Descripción: Lógica de negocio para proyectos (multi-tenant)
// Origen.....: oProyectos.cs
//
// DAL que usa:
//   crate::dal::proyectos::alta
//   crate::dal::proyectos::baja
//   crate::dal::proyectos::cambio
//   crate::dal::proyectos::asignaciones_set
//   crate::dal::proyectos::asignaciones_lst
//   crate::dal::proyectos::consulta
//   crate::dal::proyectos::llena_proyectos
//   crate::dal::proyectos::cliente_proyecto
//   crate::dal::proyectos::dir_proyecto
//   crate::dal::proyectos::total_ppto

use crate::dal::proyectos as dal;
use crate::domain::models::lookup::LookupItem;
use crate::domain::models::proyectos::{ProyectoAsignacion, Proyectos};
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn alta(pool: &PgPool, proy: &Proyectos, tenant_id: Uuid) -> ReturnCode {
    dal::alta(pool, proy, tenant_id).await
}

pub async fn baja(pool: &PgPool, id: i32, tenant_id: Uuid) -> ReturnCode {
    dal::baja(pool, id, tenant_id).await
}

pub async fn cambio(pool: &PgPool, proy: &Proyectos, tenant_id: Uuid) -> ReturnCode {
    dal::cambio(pool, proy, tenant_id).await
}

pub async fn asignaciones_set(
    pool: &PgPool, proyecto: i32, tenant_id: Uuid, grupos: &[i32], usuarios: &[i32],
) -> ReturnCode {
    dal::asignaciones_set(pool, proyecto, tenant_id, grupos, usuarios).await
}

pub async fn asignaciones_lst(
    pool: &PgPool, proyecto: i32, tenant_id: Uuid,
) -> Result<Vec<ProyectoAsignacion>, ReturnCode> {
    dal::asignaciones_lst(pool, proyecto, tenant_id).await
}

pub async fn consulta(pool: &PgPool, id: i32, tenant_id: Uuid) -> Result<Option<Proyectos>, ReturnCode> {
    dal::consulta(pool, id, tenant_id).await
}

/// Clona un proyecto COMPLETO: header (con `nombre` nuevo y, opcionalmente, otro
/// `cliente`) + todo el árbol WBS de partidas/tareas + las asignaciones grupo/usuario.
/// Respeta el tope de proyectos del plan (el alta devuelve -20 si se excede).
/// La copia del árbol y las asignaciones es best-effort: si falla, el proyecto ya
/// quedó creado (se loguea). No copia finanzas ni archivos (son transaccionales).
pub async fn clonar(
    pool: &PgPool,
    origen_id: i32,
    nombre: &str,
    cliente: Option<i32>,
    tenant_id: Uuid,
) -> Result<i32, ReturnCode> {
    // 1) Proyecto origen (consulta valida que pertenezca al tenant).
    let mut nuevo = match dal::consulta(pool, origen_id, tenant_id).await? {
        Some(p) => p,
        None => return Err(ReturnCode { codigo: -41, afectado: 0, mensaje: "El proyecto a clonar no existe".to_string() }),
    };

    // 2) Lo único que cambia: nombre (obligatorio) y, si se indicó, el cliente.
    nuevo.id     = 0;
    nuevo.nombre = nombre.to_string();
    if let Some(c) = cliente {
        nuevo.cliente = c;
    }

    // 3) Crear el header del clon (respeta el límite del plan → -20).
    let ret = dal::alta(pool, &nuevo, tenant_id).await;
    if ret.afectado <= 0 {
        return Err(ret);
    }
    let nuevo_id = ret.afectado;

    // 4) Copiar TODO el árbol WBS (el destino está vacío → copia todas las partidas).
    let wbs = crate::dal::detalle_proyecto::adiciona_partidas_faltantes(pool, origen_id, nuevo_id).await;
    if wbs.codigo < 0 {
        tracing::warn!(origen = origen_id, nuevo = nuevo_id, codigo = wbs.codigo, "clonar: árbol WBS con error (proyecto ya creado)");
    }

    // 5) Copiar las asignaciones grupo/usuario del origen.
    if let Ok(asigs) = dal::asignaciones_lst(pool, origen_id, tenant_id).await {
        if !asigs.is_empty() {
            let grupos:   Vec<i32> = asigs.iter().map(|a| a.gn_id).collect();
            let usuarios: Vec<i32> = asigs.iter().map(|a| a.gn_usr_id).collect();
            let _ = dal::asignaciones_set(pool, nuevo_id, tenant_id, &grupos, &usuarios).await;
        }
    }

    Ok(nuevo_id)
}

pub async fn llena_proyectos(
    pool: &PgPool, activos: bool, tenant_id: Uuid, grupo: i32, gn_usr_id: i32, nivel: i32,
) -> Result<Vec<Proyectos>, ReturnCode> {
    dal::llena_proyectos(pool, activos, tenant_id, grupo, gn_usr_id, nivel).await
}

pub async fn cliente_proyecto(pool: &PgPool, proyecto: i32) -> Result<String, ReturnCode> {
    dal::cliente_proyecto(pool, proyecto).await
}

pub async fn dir_proyecto(pool: &PgPool, proyecto: i32) -> Result<String, ReturnCode> {
    dal::dir_proyecto(pool, proyecto).await
}

pub async fn total_ppto(pool: &PgPool, proyecto: i32) -> Result<Decimal, ReturnCode> {
    dal::total_ppto(pool, proyecto).await
}

/// (usados, max) — proyectos activos del tenant y tope del plan (None = ilimitado).
pub async fn cupo(pool: &PgPool, tenant_id: Uuid) -> Result<(i64, Option<i32>), ReturnCode> {
    dal::cupo(pool, tenant_id).await
}

pub async fn lista_grupos(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Vec<crate::domain::models::gn_grupos::GnGrupos>, ReturnCode> {
    crate::dal::gn_grupos::obtiene_todo(pool, true, tenant_id).await
}

pub async fn usuarios_grupo(
    pool: &PgPool,
    grupo_id: i32,
    tenant_id: Uuid,
) -> Result<Vec<crate::domain::models::gn_usuarios::GnUsuarios>, ReturnCode> {
    let todos = crate::dal::gn_usuarios::obtiene_todo(pool, tenant_id).await?;
    Ok(todos.into_iter().filter(|u| u.grupo_negocio == grupo_id).collect())
}

// ─────────────────────────────────────────────
// LOOKUP — autocomplete proyectos activos
// Etiqueta del SP: "<nombre proyecto> — <cliente>"
// ─────────────────────────────────────────────
pub async fn lookup(pool: &PgPool, q: &str, cliente: Option<i32>, limit: i32, tenant_id: Uuid) -> Result<Vec<LookupItem>, ReturnCode> {
    dal::lookup(pool, q, cliente, limit, tenant_id).await
}
