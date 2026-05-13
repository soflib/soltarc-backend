// Programa...: services::reportes::financieros
// Descripción: Reportes financieros (ingresos, egresos, captura diaria)
// Origen.....: oReportes.cs

use crate::dal::reportes as dal;
use crate::domain::models::reportes::{
    CapturaDiaria, EgresosCentroCosto, EgresosProveedor, EgresosReporte, IngresosCliente, IngresosReporte,
};
use crate::infrastructure::db::return_code::ReturnCode;
use sqlx::PgPool;
use time::Date;

pub async fn captura_diaria(pool: &PgPool, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<CapturaDiaria>, ReturnCode> {
    dal::captura_diaria(pool, fecha_ini, fecha_fin).await
}

pub async fn ingresos_reporte(pool: &PgPool, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<IngresosReporte>, ReturnCode> {
    dal::ingresos_reporte(pool, fecha_ini, fecha_fin).await
}

pub async fn ingresos_cliente(pool: &PgPool, id: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<IngresosCliente>, ReturnCode> {
    dal::ingresos_cliente(pool, id, fecha_ini, fecha_fin).await
}

pub async fn egresos_centros_costo(pool: &PgPool, id: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosCentroCosto>, ReturnCode> {
    dal::egresos_centros_costo(pool, id, fecha_ini, fecha_fin).await
}

pub async fn egresos_proveedor(pool: &PgPool, id: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosProveedor>, ReturnCode> {
    dal::egresos_proveedor(pool, id, fecha_ini, fecha_fin).await
}

pub async fn egresos_reporte(pool: &PgPool, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosReporte>, ReturnCode> {
    dal::egresos_reporte(pool, fecha_ini, fecha_fin).await
}

pub async fn reporte_gral_egresos(pool: &PgPool, banco: i32, fecha_ini: Date, fecha_fin: Date) -> Result<Vec<EgresosReporte>, ReturnCode> {
    dal::reporte_gral_egresos(pool, banco, fecha_ini, fecha_fin).await
}
