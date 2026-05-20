// Programa...: services::operaciones::detalle_proyecto
// Descripción: Lógica de negocio para detalle de proyectos
// Origen.....: oDetalleProyecto.cs
//
// DAL que usa:
//   crate::dal::detalle_proyecto::*

use crate::dal::detalle_proyecto as dal;
use crate::domain::models::detalle_proyectos::{DetalleProyectos, NodoArbol};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub async fn alta(pool: &PgPool, det: &DetalleProyectos) -> ReturnCode {
    dal::alta(pool, det).await
}

pub async fn baja(pool: &PgPool, id: i32) -> ReturnCode {
    dal::baja(pool, id).await
}

pub async fn cambios(pool: &PgPool, det: &DetalleProyectos) -> ReturnCode {
    dal::cambios(pool, det).await
}

pub async fn consulta(pool: &PgPool, id: i32) -> Result<Option<DetalleProyectos>, ReturnCode> {
    dal::consulta(pool, id).await
}

pub async fn partidas_proyecto(pool: &PgPool, proyecto: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    dal::partidas_proyecto(pool, proyecto).await
}

pub async fn carga_tareas(pool: &PgPool, proyecto: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    dal::carga_tareas(pool, proyecto).await
}

pub async fn nodos_desc(pool: &PgPool, nodo_raiz: &str) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    dal::nodos_desc(pool, nodo_raiz).await
}

pub async fn actualiza_fechas(
    pool: &PgPool,
    nodo: &str,
    fecha_ini: Date,
    fecha_fin: Date,
    estado: i32,
    fecha_termino: Date,
) -> ReturnCode {
    dal::actualiza_fechas(pool, nodo, fecha_ini, fecha_fin, estado, fecha_termino).await
}

pub async fn copia_contenido_partidas(pool: &PgPool, origen: i32, destino: i32) -> ReturnCode {
    dal::copia_contenido_partidas(pool, origen, destino).await
}

pub async fn adiciona_partidas_faltantes(pool: &PgPool, origen: i32, destino: i32) -> ReturnCode {
    dal::adiciona_partidas_faltantes(pool, origen, destino).await
}

pub async fn adiciona_partidas_qry(pool: &PgPool, origen: i32, destino: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    dal::adiciona_partidas_qry(pool, origen, destino).await
}

pub async fn copia_cont_partidas_qry(pool: &PgPool, origen: i32, destino: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    dal::copia_cont_partidas_qry(pool, origen, destino).await
}

pub async fn carga_nivel(pool: &PgPool, proyecto: i32, nivel: i32) -> Result<Vec<DetalleProyectos>, ReturnCode> {
    let lista = dal::carga_tareas(pool, proyecto).await?;
    let filtrada: Vec<DetalleProyectos> = lista.into_iter()
        .filter(|d| d.nodo.split('/').filter(|s| !s.is_empty()).count() as i32 == nivel)
        .collect();
    if filtrada.is_empty() {
        Err(ReturnCode { codigo: -70, afectado: 0, mensaje: "No hay partidas en ese nivel".to_string() })
    } else {
        Ok(filtrada)
    }
}

// ─────────────────────────────────────────────
// ARBOL — árbol WBS del proyecto con id, nodo, descripcion, nivel, importe, estado.
// Reemplaza la necesidad de combinar partidas_proyecto + carga_tareas
// para construir el árbol en el frontend.
// ─────────────────────────────────────────────
pub async fn arbol(pool: &PgPool, proyecto: i32) -> Result<Vec<NodoArbol>, ReturnCode> {
    dal::arbol(pool, proyecto).await
}

// ─────────────────────────────────────────────
// BUSCAR — búsqueda por descripción dentro del WBS del proyecto, devolviendo
// `ruta` con las descripciones ancestrales concatenadas.
// ─────────────────────────────────────────────
pub async fn buscar(pool: &PgPool, proyecto: i32, texto: &str) -> Result<Vec<NodoArbol>, ReturnCode> {
    dal::buscar(pool, proyecto, texto).await
}

// ── CSV import ────────────────────────────────────────────────────────────────
// Columnas esperadas (con encabezado):
//   tipo,secuencia,descripcion,comentarios,presupuesto,fecha_inicio,fecha_fin,fecha_termina,estado,nodo
// El proyecto_id viene del parámetro de la petición, no del CSV.
// Fechas en formato: YYYY-MM-DD HH:MM:SS
// Campos de texto con comas deben ir entre comillas dobles.

fn csv_split(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => { fields.push(field.trim().to_string()); field = String::new(); }
            _ => field.push(ch),
        }
    }
    fields.push(field.trim().to_string());
    fields
}

pub async fn import_csv(
    pool: &PgPool,
    proyecto_id: i32,
    csv_bytes: &[u8],
) -> (i32, Vec<String>) {
    use chrono::NaiveDateTime;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let text = match std::str::from_utf8(csv_bytes) {
        Ok(t) => t,
        Err(_) => return (0, vec!["El archivo no es UTF-8 válido".to_string()]),
    };

    let mut inserted = 0i32;
    let mut errors: Vec<String> = Vec::new();

    let parse_dt = |s: &str| -> Result<NaiveDateTime, String> {
        NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(s.trim(), "%Y-%m-%d"))
            .map_err(|_| format!("fecha inválida: '{}'", s))
    };

    for (row_idx, line) in text.lines().enumerate() {
        if row_idx == 0 || line.trim().is_empty() { continue; } // skip header

        let cols = csv_split(line);
        if cols.len() < 9 {
            errors.push(format!("Fila {}: se esperaban 9 columnas, se recibieron {}", row_idx + 1, cols.len()));
            continue;
        }

        let tipo      = cols[0].parse::<i32>().unwrap_or(1);
        let secuencia = cols[1].parse::<i32>().unwrap_or(0);
        let descripcion  = cols[2].clone();
        let comentarios  = cols[3].clone();
        let presupuesto  = Decimal::from_str(&cols[4]).unwrap_or(Decimal::ZERO);
        let fecha_inicio = match parse_dt(&cols[5]) { Ok(d) => d, Err(e) => { errors.push(format!("Fila {}: {}", row_idx + 1, e)); continue; } };
        let fecha_fin    = match parse_dt(&cols[6]) { Ok(d) => d, Err(e) => { errors.push(format!("Fila {}: {}", row_idx + 1, e)); continue; } };
        let fecha_termina = match parse_dt(&cols[7]) { Ok(d) => d, Err(e) => { errors.push(format!("Fila {}: {}", row_idx + 1, e)); continue; } };
        let estado = cols[8].parse::<i32>().unwrap_or(0);
        let nodo   = if cols.len() > 9 { cols[9].clone() } else { "/".to_string() };

        let det = DetalleProyectos {
            id: 0,
            proyecto: proyecto_id,
            tipo,
            secuencia,
            descripcion,
            comentarios,
            presupuesto,
            fecha_inicio,
            fecha_fin,
            fecha_termina,
            estado,
            nodo,
        };

        let ret = dal::alta(pool, &det).await;
        if ret.afectado > 0 {
            inserted += 1;
        } else {
            errors.push(format!("Fila {}: {} (código {})", row_idx + 1, ret.mensaje, ret.codigo));
        }
    }

    (inserted, errors)
}
