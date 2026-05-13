// Programa...: models/finanzas.rs
// Descripción: Modelos de dominio para los procesos financieros
//
// ─────────────────────────────────────────────────────────────────────────────
// POR QUÉ ESTOS MODELOS NO EXISTÍAN EN C#
// ─────────────────────────────────────────────────────────────────────────────
// En C# todos los métodos de oFinanzas devolvían DataSet, que es un contenedor
// genérico sin tipo. Las columnas nunca se declaraban en una clase — vivían
// implícitamente en el GridView del ASPX o en funciones LlenaTabla() donde
// se accedía por índice (RengP.ItemArray[0], [1], etc.).
//
// En Rust sqlx necesita mapear cada fila a un struct concreto en tiempo de
// compilación. Por eso estos modelos se crean ahora, inferidos de:
//
//   • BuscaTRX.aspx.cs      → columnas del SELECT inline de egresos
//   • EgresosProveedor.cs   → LlenaTabla() índices 0-5
//   • EgresosProveedorProyecto.cs → comentario "0=Tipo,1=Proveedor,2=Proyecto,3=Monto"
//   • CapturaDiaria.cs      → RowDataBound con "EGT"/"INT" y 13 columnas
//   • oFinanzas.cs          → parámetros y nombres de SPs
//
// Todos los campos están marcados con su origen para que puedas ajustarlos
// cuando tengas acceso al SP real en PostgreSQL.
// ─────────────────────────────────────────────────────────────────────────────

use rust_decimal::Decimal;
use time::Date;
use utoipa::ToSchema;
// ─────────────────────────────────────────────
// TrxFinanciera
// Origen: sp_cpa_FinanzasProyQry(@Proyecto)
//
// Propósito: Muestra todas las transacciones financieras (ingresos Y egresos)
//            de un proyecto en una sola vista consolidada. Se usa en la
//            pantalla principal de finanzas del proyecto para ver el flujo
//            completo de dinero: qué entró, qué salió y cuándo.
//
// Inferido de: BuscaTRX.aspx.cs — SELECT inline con JOIN a cpa_egresos,
//              cpa_Proyectos, cpa_Catalogos y aspnet_Users
// ─────────────────────────────────────────────
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct TrxFinanciera {
    /// Identificador único de la transacción (egr_Id / ing_Id)
    pub id: i32,                        // inferido de egr_Id

    /// "EGR" = egreso, "ING" = ingreso — discrimina el tipo de movimiento
    pub tipo: String,                   // inferido del discriminador EGT/INT en CapturaDiaria

    /// Fecha en que se capturó la transacción
    pub fecha_cap: Option<Date>,        // inferido de egr_Fecha / CONVERT(varchar,[egr_Fecha],11)

    /// Fecha en que el movimiento aplica contablemente
    pub fecha_aplica: Option<Date>,     // inferido de egr_FechaAplica

    /// Nombre del banco (viene del JOIN con cpa_Catalogos)
    pub banco: Option<String>,          // inferido de cen_Nombre AS 'Banco'

    /// Número o clave de la cuenta bancaria
    pub cuenta: Option<String>,         // inferido de egr_Cuenta

    /// Forma de pago: transferencia, cheque, efectivo, etc.
    pub forma_pago: Option<String>,     // inferido de egr_FormaPago

    /// Id del centro de costo asociado
    pub centro_costo: Option<i32>,      // inferido de egr_CentroCosto AS 'Centro'

    /// Referencia del movimiento (número de cheque, folio, etc.)
    pub referencia: Option<String>,     // inferido de egr_Referencia

    /// Comentario libre de la transacción
    pub comentario: Option<String>,     // inferido de egr_Comentario AS 'Comentarios'

    /// Nombre del proyecto al que pertenece
    pub proyecto: Option<String>,       // inferido de pry_Nombre AS 'Proyecto'

    /// Nombre del usuario que capturó (viene de aspnet_Users)
    pub usuario: Option<String>,        // inferido de UserName AS 'Usuario'

    /// Id del proveedor o cliente relacionado
    pub cte_pro: Option<i32>,           // inferido de egr_Proveedor AS 'Cte/Pro'

    /// Importe del movimiento
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,         // inferido de egr_Monto / CONVERT(varchar,[egr_Monto],1)
}

// ─────────────────────────────────────────────
// ResumenProyecto
// Origen: sp_cpa_FinanzasProySum(@GN_Id, @Usr_Id, @Usr_Nivel)
//
// Propósito: Dashboard financiero por grupo de negocio. Muestra todos los
//            proyectos visibles para el usuario (según su nivel de acceso)
//            con sus totales de presupuesto, ingresos y egresos. Es la vista
//            de "cómo van mis proyectos en dinero" del panel principal.
//
// Inferido de: LlenaDetProyectos() — recibe Grupo, Usuario, Nivel=5
//              y su resultado se ligaba directamente a un GridView de resumen
// ─────────────────────────────────────────────
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct ResumenProyecto {
    /// Id del proyecto
    pub proyecto_id: i32,               // inferido de pry_Id

    /// Nombre del proyecto
    pub nombre: Option<String>,         // inferido de pry_Nombre

    /// Presupuesto original asignado al proyecto
    #[schema(value_type = Option<f64>)]
    pub presupuesto: Option<Decimal>,   // inferido de pry_Presupuesto / Money

    /// Suma de todos los ingresos registrados del proyecto
    #[schema(value_type = Option<f64>)]
    pub total_ingresos: Option<Decimal>, // inferido — agregado del SP

    /// Suma de todos los egresos registrados del proyecto
    #[schema(value_type = Option<f64>)]
    pub total_egresos: Option<Decimal>, // inferido — agregado del SP

    /// Saldo: presupuesto - egresos + ingresos
    #[schema(value_type = Option<f64>)]
    pub saldo: Option<Decimal>,         // inferido — calculado en el SP

    /// Estado del proyecto (FK a cpa_Catalogos tipo 1)
    pub estado: Option<i32>,            // inferido de pry_Estado

    /// Nombre del cliente del proyecto
    pub cliente: Option<String>,        // inferido — JOIN con cpa_Clientes
}

// ─────────────────────────────────────────────
// EgresosProveedorProyecto
// Origen: sp_cpa_EgresosProveedorProyecto(@TipoRep, @FechaIni, @FechaFin)
//
// Propósito: Reporte cruzado proveedor × proyecto. Según el flag TipoRep
//            agrupa primero por proveedor (false) o primero por proyecto (true).
//            Permite responder "¿cuánto le pagamos al proveedor X por proyecto?"
//            o "¿cuánto gastó el proyecto Y por proveedor?".
//
// Inferido de: EgresosProveedorProyecto.aspx.cs — comentario explícito:
//              "0=Tipo, 1=Proveedor, 2=Proyecto, 3=Monto"
//              oTota.Numero_Columna_Corte_1 = 1 (corte por proveedor)
//              oTota.Columna_Monto = 3
// ─────────────────────────────────────────────
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct EgresosProveedorProyecto {
    /// Discriminador para subtotales: "DAT"=dato, "TOT"=total — usado por la capa de presentación
    pub tipo: Option<String>,           // inferido de columna 0 — "Tipo"

    /// Nombre del proveedor
    pub proveedor: Option<String>,      // inferido de columna 1 — "Proveedor"

    /// Nombre del proyecto
    pub proyecto: Option<String>,       // inferido de columna 2 — "Proyecto"

    /// Importe del egreso
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,         // inferido de columna 3 — "Monto"
}

// ─────────────────────────────────────────────
// IngresosDetalle
// Origen: sp_cpa_IngresosQryGeneral(@FechaIni, @FechaFin)
//
// Propósito: Detalle completo de todos los ingresos en un rango de fechas,
//            sin filtro por proyecto ni cliente. Se usa en reportes generales
//            de tesorería para ver el dinero que entró al negocio en un período.
//            Es el equivalente de egresos de BuscaTRX pero para ingresos.
//
// Inferido de: oIngresos.cs — campos de eIngresos + nombres de parámetros del SP
//              Los campos son simétricos a TrxFinanciera pero sin centro_costo
//              ni proveedor, y con cliente en su lugar
// ─────────────────────────────────────────────
#[derive(Debug, ToSchema, sqlx::FromRow)]
pub struct IngresosDetalle {
    /// Identificador único del ingreso
    pub id: i32,                        // inferido de ing_Id

    /// Fecha de captura del ingreso
    pub fecha: Option<Date>,            // inferido de ing_Fecha

    /// Id del banco donde se recibió el ingreso
    pub banco: Option<i32>,             // inferido de ing_Banco

    /// Número o clave de la cuenta bancaria receptora
    pub cuenta: Option<String>,         // inferido de ing_Cuenta

    /// Forma en que se recibió el pago
    pub forma_pago: Option<String>,     // inferido de ing_FormaPago

    /// Id del proyecto al que aplica el ingreso
    pub proyecto: Option<i32>,          // inferido de ing_Proyecto

    /// Importe recibido
    #[schema(value_type = Option<f64>)]
    pub monto: Option<Decimal>,         // inferido de ing_Monto

    /// Referencia del pago (número de transferencia, cheque, etc.)
    pub referencia: Option<String>,     // inferido de ing_Referencia

    /// Comentario libre
    pub comentario: Option<String>,     // inferido de ing_Comentario

    /// Fecha en que el ingreso aplica contablemente
    pub fecha_aplica: Option<Date>,     // inferido de ing_FechaAplica

    /// Id del cliente que realizó el pago
    pub cliente: Option<i32>,           // inferido de ing_Cliente
}