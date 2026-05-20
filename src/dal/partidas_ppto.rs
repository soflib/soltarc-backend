// Programa...: partidas_ppto
// Descripción: Operaciones de la tabla ppto_Partidas_PPTO
// Origen.....: oPartidasPPTO.cs
//
// Stored Procedures que usa:
//   ppto_sp_PartidasPPTO_Add     → alta
//   ppto_sp_PartidasPPTO_DEL     → borra (con nodo)
//   ppto_sp_PartidasPPTO_UPD     → cambio
//   ppto_sp_PartidasPPTO_LSTPPTO → carga partidas por presupuesto
//   ppto_sp_PartidasNODOS_UPD    → actualiza nodo
//   ppto_sp_PartidasPPTO_SigNod  → calcula siguiente nodo hijo
//   ppto_sp_PartidasPPTO_QRY2LVL → carga partidas de 2° nivel
//
// Nota: Nuevo_Nodo_Adiciona() contenía lógica de cadenas pura en C#.
//       Se migra a Rust manteniendo la misma lógica sin SP adicional.
//       Carga2Nivel() devuelve Vec<PartidasPpto> en lugar de llenar un ListBox.

use crate::domain::models::partidas_ppto::{PartidaBuscada, PartidasPpto};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;

// ─────────────────────────────────────────────
// ALTA — ppto_sp_PartidasPPTO_Add
// ─────────────────────────────────────────────
pub async fn alta(pool: &PgPool, par: &PartidasPpto) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_PartidasPPTO_Add($1,$2,$3,$4,$5,$6,$7)"
    )
    .bind(par.presupuesto)
    .bind(&par.nodo)
    .bind(&par.concepto)
    .bind(par.unidad)
    .bind(par.cantidad)
    .bind(par.precio_u)
    .bind(par.calculo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(id) if id > 0 => ReturnCode { codigo: 10,  afectado: id, mensaje: "Alta realizada Ok".to_string() },
        Ok(_)            => ReturnCode { codigo: -11, afectado: 0,  mensaje: "Alta cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -15, afectado: 0,  mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// BORRA — ppto_sp_PartidasPPTO_DEL
// Retornos especiales: -1 = tiene hijos, 0 = cancelada, >0 = ok
// ─────────────────────────────────────────────
pub async fn borra(pool: &PgPool, id: i32, nodo: &str) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_PartidasPPTO_DEL($1, $2)"
    )
    .bind(id)
    .bind(nodo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(-1)        => ReturnCode { codigo: -21, afectado: 0,  mensaje: "Tiene hijos, baja inválida".to_string() },
        Ok(n) if n > 0 => ReturnCode { codigo: 20,  afectado: n, mensaje: format!("La partida: {} se borró ok", id) },
        Ok(_)          => ReturnCode { codigo: -22, afectado: 0, mensaje: "Baja cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -25, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CAMBIO — ppto_sp_PartidasPPTO_UPD
// ─────────────────────────────────────────────
pub async fn cambio(pool: &PgPool, par: &PartidasPpto) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_PartidasPPTO_UPD($1,$2,$3,$4,$5,$6)"
    )
    .bind(par.id.unwrap_or(0))  // id es Option<i32> — 0 nunca debería llegar aquí
    .bind(&par.concepto)
    .bind(par.unidad)
    .bind(par.cantidad)
    .bind(par.precio_u)
    .bind(par.calculo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0 => ReturnCode { codigo: 30,  afectado: n, mensaje: format!("Nodo: {} se actualizó ok", par.id.unwrap_or(0)) },
        Ok(_)          => ReturnCode { codigo: -31, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)         => ReturnCode { codigo: -35, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// CARGA PARTIDAS — ppto_sp_PartidasPPTO_LSTPPTO
// ─────────────────────────────────────────────
pub async fn carga_partidas(pool: &PgPool, presupuesto: i32) -> Result<Vec<PartidasPpto>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidasPpto>(
        "SELECT * FROM arqeth.ppto_sp_PartidasPPTO_LSTPPTO($1)"
    )
    .bind(presupuesto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -110, afectado: 0, mensaje: "No hay partidas en el presupuesto, se requiere al menos (1)".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -115, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// ACTUALIZA NODO — ppto_sp_PartidasNODOS_UPD
// Retornos especiales: -5 = tiene hijos
// ─────────────────────────────────────────────
pub async fn partidas_actualiza_nodo(pool: &PgPool, id: i32, ppto: i32, nuevo_nodo: &str) -> ReturnCode {
    let result = sqlx::query_scalar::<_, i32>(
        "SELECT arqeth.ppto_sp_PartidasNODOS_UPD($1, $2, $3)"
    )
    .bind(id)
    .bind(ppto)
    .bind(nuevo_nodo)
    .fetch_one(pool)
    .await;

    match result {
        Ok(n) if n > 0  => ReturnCode { codigo: 40,  afectado: n, mensaje: "Actualización ok".to_string() },
        Ok(-5)           => ReturnCode { codigo: -41, afectado: 0, mensaje: "Actualización cancelada, tiene hijos".to_string() },
        Ok(_)            => ReturnCode { codigo: -42, afectado: 0, mensaje: "Actualización cancelada".to_string() },
        Err(e)           => ReturnCode { codigo: -45, afectado: 0, mensaje: e.to_string() },
    }
}

// ─────────────────────────────────────────────
// NUEVO NODO ADICIONA — ppto_sp_PartidasPPTO_SigNod
// Calcula el siguiente nodo hijo. La lógica de construcción
// de cadena del nodo se mantiene igual que en C#.
// Devuelve Ok(nodo_string) o Err(ReturnCode).
// ─────────────────────────────────────────────
pub async fn nuevo_nodo_adiciona(
    pool: &PgPool,
    ppto: i32,
    nodo: &str,
    nivel: i32,
) -> Result<String, ReturnCode> {
    let result = sqlx::query_scalar::<_, Option<String>>(
        "SELECT arqeth.ppto_sp_PartidasPPTO_SigNod($1, $2, $3)"
    )
    .bind(ppto)
    .bind(nodo)
    .bind(nivel)
    .fetch_optional(pool)
    .await;

    match result {
        Err(e) => Err(ReturnCode { codigo: -86, afectado: 0, mensaje: e.to_string() }),
        Ok(None) | Ok(Some(None)) => {
            // No hay hijos — primer hijo del nodo
            Ok(format!("{}1/", nodo))
        }
        Ok(Some(Some(nodo_encontrado))) => {
            // Quitar slash final si aplica
            let nodo_final = if nodo_encontrado != nodo && nodo_encontrado.ends_with('/') {
                nodo_encontrado[..nodo_encontrado.len() - 1].to_string()
            } else {
                nodo_encontrado.clone()
            };

            // Calcular posición y valor del último hijo
            let pos_valor = nodo.len();
            let sufijo = &nodo_final[pos_valor..];
            let val_nuevo_hijo: i32 = if sufijo.is_empty() {
                1
            } else {
                sufijo.trim_end_matches('/')
                    .parse::<i32>()
                    .unwrap_or(0) + 1
            };

            Ok(format!("{}{}/", nodo, val_nuevo_hijo))
        }
    }
}

// ─────────────────────────────────────────────
// CARGA 2° NIVEL — ppto_sp_PartidasPPTO_QRY2LVL
// Devuelve Vec<PartidasPpto> en lugar de llenar un ListBox
// ─────────────────────────────────────────────
pub async fn carga_2_nivel(pool: &PgPool, nodo: i32, ppto: i32) -> Result<Vec<PartidasPpto>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidasPpto>(
        "SELECT * FROM arqeth.ppto_sp_PartidasPPTO_QRY2LVL($1, $2)"
    )
    .bind(nodo)
    .bind(ppto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) if !lista.is_empty() => Ok(lista),
        Ok(_)  => Err(ReturnCode { codigo: -91, afectado: 0, mensaje: "No hay partidas del nivel".to_string() }),
        Err(e) => Err(ReturnCode { codigo: -96, afectado: 0, mensaje: e.to_string() }),
    }
}

// ─────────────────────────────────────────────
// BUSCAR — sp_cpa_partidasppto_buscar
// Búsqueda por concepto dentro de un presupuesto. Cada fila incluye `ruta`
// con los conceptos ancestros ("Preliminares > Excavación").
// Para el árbol completo, sigue usándose `carga_partidas`.
// ─────────────────────────────────────────────
pub async fn buscar(pool: &PgPool, presupuesto: i32, texto: &str) -> Result<Vec<PartidaBuscada>, ReturnCode> {
    let result = sqlx::query_as::<_, PartidaBuscada>(
        "SELECT * FROM arqeth.sp_cpa_partidasppto_buscar($1, $2)"
    )
    .bind(presupuesto)
    .bind(texto)
    .fetch_all(pool)
    .await;

    match result {
        Ok(lista) => Ok(lista),
        Err(e)    => Err(ReturnCode { codigo: -125, afectado: 0, mensaje: e.to_string() }),
    }
}
