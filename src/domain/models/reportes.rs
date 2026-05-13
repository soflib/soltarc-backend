// Programa...: models/reportes.rs
// Descripción: Modelos de dominio para los reportes del sistema
// Origen.....: oReportes.cs — structs inferidos de los SPs y vistas C#

use rust_decimal::Decimal;
use time::Date;
use utoipa::ToSchema;

// CapturaDiaria — sp_cpa_CapturaDiaria(@FechaIni, @FechaFin)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct CapturaDiaria {
    pub tipo: Option<String>,
    pub fecha: Option<Date>,
    pub banco: Option<String>,
    pub cuenta: Option<String>,
    pub referencia: Option<String>,
    pub concepto: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub usuario: Option<String>,
    pub proyecto: Option<String>,
}

// IngresosReporte — sp_cpa_IngresosReporte(@FechaIni, @FechaFin)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct IngresosReporte {
    pub id: Option<i32>,
    pub fecha: Option<Date>,
    pub banco: Option<String>,
    pub cuenta: Option<String>,
    pub forma_pago: Option<String>,
    pub referencia: Option<String>,
    pub cliente: Option<String>,
    pub proyecto: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub comentario: Option<String>,
}

// IngresosCliente — sp_cpa_IngresosQryCliente(@Id, @FechaIni, @FechaFin)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct IngresosCliente {
    pub id: Option<i32>,
    pub fecha: Option<Date>,
    pub banco: Option<String>,
    pub cuenta: Option<String>,
    pub forma_pago: Option<String>,
    pub referencia: Option<String>,
    pub proyecto: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub comentario: Option<String>,
}

// EstadoCuenta — sp_cpa_IngresosEstadoCuenta(@Id)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct EstadoCuenta {
    pub fecha: Option<Date>,
    pub concepto: Option<String>,
    pub referencia: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub cargo: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub abono: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub saldo: Option<Decimal>,
}

// PartidasPptoReporte — ppto_sp_Reportes_PPTO(@presupuesto)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct PartidasPptoReporte {
    pub nodo: Option<String>,
    pub concepto: Option<String>,
    pub unidad: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub cantidad: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub precio_u: Option<Decimal>,
    #[schema(value_type = Option<f64>)]
    pub importe: Option<Decimal>,
    pub calculo: Option<i32>,
    pub nivel: Option<i32>,
}

// PartidasArbol — sp_cpa_GpoDetProysArbol / sp_cpa_Audita_XREF
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct PartidasArbol {
    pub nodo: Option<String>,
    pub nivel: Option<i32>,
    pub descripcion: Option<String>,
    pub estado: Option<i32>,
    pub proyecto: Option<i32>,
    #[schema(value_type = Option<f64>)]
    pub importe: Option<Decimal>,
}

// RegistroAvance — sp_cpa_Proy_AvanceDeObraIng / sp_cpa_Proy_AvanceDeObraEgr
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct RegistroAvance {
    pub fecha: Option<Date>,
    pub concepto: Option<String>,
    pub referencia: Option<String>,
    pub proyecto: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub usuario: Option<String>,
}

// EgresosCentroCosto — sp_cpa_EgresosQryCenCo(@Id, @FechaIni, @FechaFin)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct EgresosCentroCosto {
    pub id: Option<i32>,
    pub fecha: Option<Date>,
    pub banco: Option<String>,
    pub cuenta: Option<String>,
    pub forma_pago: Option<String>,
    pub referencia: Option<String>,
    pub proyecto: Option<String>,
    pub proveedor: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub comentario: Option<String>,
}

// EgresosProveedor — sp_cpa_EgresosQryProveedor(@Id, @FechaIni, @FechaFin)
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct EgresosProveedor {
    pub id: Option<i32>,
    pub fecha: Option<Date>,
    pub banco: Option<String>,
    pub cuenta: Option<String>,
    pub forma_pago: Option<String>,
    pub referencia: Option<String>,
    pub proyecto: Option<String>,
    pub centro_costo: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub comentario: Option<String>,
}

// EgresosReporte — sp_cpa_FinanzasEgresosAcum / sp_cpa_Reporte_Egresos
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct EgresosReporte {
    pub id: Option<i32>,
    pub fecha: Option<Date>,
    pub banco: Option<String>,
    pub cuenta: Option<String>,
    pub forma_pago: Option<String>,
    pub referencia: Option<String>,
    pub proyecto: Option<String>,
    pub proveedor: Option<String>,
    pub centro_costo: Option<String>,
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,
    pub comentario: Option<String>,
    pub usuario: Option<String>,
}
