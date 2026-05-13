// Programa...: services::finanzas::finanzas_reports
// Origen.....: oFinanzas.cs + ReciboHonorarios.aspx

use crate::dal::{egresos as dal_egresos, finanzas, ingresos as dal_ingresos};
use crate::domain::models::finances::{EgresosProveedorProyecto, IngresosDetalle, TrxFinanciera};
use crate::infrastructure::db::return_code::ReturnCode;
use rust_decimal::Decimal;
use sqlx::PgPool;
use time::Date;

pub async fn trx_financieras(pool: &PgPool, proyecto: i32) -> Result<Vec<TrxFinanciera>, ReturnCode> {
    finanzas::trx_financieras(pool, proyecto).await
}

pub async fn distribuye_egreso(pool: &PgPool, egreso: i32, nodo: &str) -> ReturnCode {
    finanzas::distribuye_egreso(pool, egreso, nodo).await
}

pub async fn egresos_proveedor_proyecto(
    pool: &PgPool,
    tipo_rep: bool,
    fecha_ini: Date,
    fecha_fin: Date,
) -> Result<Vec<EgresosProveedorProyecto>, ReturnCode> {
    finanzas::egresos_proveedor_proyecto(pool, tipo_rep, fecha_ini, fecha_fin).await
}

pub async fn ingresos_detalle(
    pool: &PgPool,
    fecha_ini: Date,
    fecha_fin: Date,
) -> Result<Vec<IngresosDetalle>, ReturnCode> {
    finanzas::ingresos_detalle(pool, fecha_ini, fecha_fin).await
}

// ── Recibo de honorarios ──────────────────────────────────────────────────────

pub struct ReciboH {
    pub egr_ing:     bool,
    pub cte_prov:    String,
    pub banco:       String,
    pub forma_pago:  String,
    pub monto:       Decimal,
    pub fecha_aplica: String,
    pub proyecto:    String,
    pub comentarios: String,
    pub referencia:  String,
    pub rfc_curp:    String,  // pendiente de join con proveedores/clientes (sin campo en SP actual)
}

pub async fn recibo_honorarios(
    pool: &PgPool,
    id: i32,
    tipo: &str,
) -> Result<ReciboH, ReturnCode> {
    match tipo {
        "egreso" => {
            let egr = dal_egresos::consulta(pool, id)
                .await?
                .ok_or_else(|| ReturnCode { codigo: -1, afectado: 0, mensaje: format!("Egreso {} no encontrado", id) })?;
            Ok(ReciboH {
                egr_ing:      true,
                cte_prov:     egr.proveedor_nombre.unwrap_or_default(),
                banco:        egr.banco_nombre.unwrap_or_default(),
                forma_pago:   egr.forma_pago,
                monto:        egr.monto,
                fecha_aplica: egr.fecha_aplica.format("%Y-%m-%d").to_string(),
                proyecto:     egr.proyecto_nombre.unwrap_or_default(),
                comentarios:  egr.comentario,
                referencia:   egr.referencia,
                rfc_curp:     String::new(),
            })
        }
        "ingreso" => {
            let ing = dal_ingresos::consulta(pool, id)
                .await?
                .ok_or_else(|| ReturnCode { codigo: -1, afectado: 0, mensaje: format!("Ingreso {} no encontrado", id) })?;
            Ok(ReciboH {
                egr_ing:      false,
                cte_prov:     ing.cliente_nombre.unwrap_or_default(),
                banco:        ing.banco_nombre.unwrap_or_default(),
                forma_pago:   ing.forma_pago,
                monto:        ing.monto,
                fecha_aplica: ing.fecha_aplica.format("%Y-%m-%d").to_string(),
                proyecto:     ing.proyecto_nombre.unwrap_or_default(),
                comentarios:  ing.comentario,
                referencia:   ing.referencia,
                rfc_curp:     String::new(),
            })
        }
        _ => Err(ReturnCode { codigo: -1, afectado: 0, mensaje: format!("Tipo inválido '{}': use 'egreso' o 'ingreso'", tipo) }),
    }
}
