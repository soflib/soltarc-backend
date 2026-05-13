// Programa...: models/partidas_ppto.rs
// Descripción: Modelo de dominio para las partidas de presupuesto
//
// ─────────────────────────────────────────────────────────────────────────────
// POR QUÉ ESTE MODELO NO EXISTÍA EN C#
// ─────────────────────────────────────────────────────────────────────────────
// En C# oPartidasPPTO.CargaPartidas() devolvía un DataSet genérico sin tipo.
// Las columnas nunca se declararon en una entidad — vivían implícitamente
// distribuidas en tres lugares diferentes del código:
//
//   1. MttoPartidas.aspx.cs (mantenimiento)
//      Accede por FindControl("lId"), FindControl("lNivel"), FindControl("lNodo"),
//      FindControl("lConcepto"), FindControl("lUnidad"), FindControl("lCantidad"),
//      FindControl("tPrecioU"), FindControl("ddCalculo") — revela los campos
//      editables del GridView.
//
//   2. MttoPartidasRepPPTO.aspx.cs y Presupuesto.aspx.cs (reportes HTML/GridView)
//      EscribeReng() accede por índice numérico al SqlDataReader:
//        DrPart.GetString(0)  → Nodo
//        DrPart.GetString(1)  → Concepto
//        DrPart.GetString(2)  → Unidad (nombre corto)
//        DrPart.GetDecimal(3) → Cantidad
//        DrPart.GetDecimal(4) → PrecioU
//        DrPart.GetInt32(6)   → Calculo
//        DrPart.GetInt32(7)   → Nivel
//      (índice 5 no se lee directamente en EscribeReng — es el importe calculado)
//
//   3. pdfPresupuesto.aspx.cs (reporte PDF con iTextSharp)
//      EscribeReng() idéntico al de arriba — confirma los mismos índices.
//      EscribePartida() usa GetInt32(6) = Calculo con valores:
//        0 = Monto fijo     → total = PrecioU
//        1 = Normal         → total = Cantidad * PrecioU
//        2 = % del padre    → total = Cantidad * (PrecioU / 100)
//        3 = % del total    → total = GranTotal * (PrecioU / 100)
//       99 = Solo título    → total = 0, sin importe
//
//   4. CambiaPartidas.aspx.cs (mantenimiento de nodos)
//      Accede a ItemArray[0] = Id, ItemArray[4] = Concepto — confirma
//      que el Id está en la columna 0 del DataSet.
//
// La columna 5 (índice 5) que no aparece en EscribeReng corresponde al
// importe pre-calculado que el SP devuelve pero el reporte recalcula
// en C# por su cuenta. Se expone aquí como `importe_calculado` para
// que el SP PostgreSQL pueda devolverlo directamente.
// ─────────────────────────────────────────────────────────────────────────────

use rust_decimal::Decimal;

// ─────────────────────────────────────────────
// PartidasPpto
// Origen: ppto_sp_PartidasPPTO_LSTPPTO(@parp_Presupuesto)
//         ppto_sp_PartidasPPTO_Add / UPD / QRY
//
// Propósito: Representa una partida dentro de un presupuesto jerárquico.
//            El presupuesto se organiza como un árbol de nodos donde cada
//            partida tiene un nodo (ruta), un nivel (profundidad en el árbol),
//            un concepto (descripción), unidad de medida, cantidad, precio
//            unitario y un tipo de cálculo que determina cómo se obtiene
//            el importe total de esa partida.
//
//            El árbol de nodos usa rutas tipo "/1/2/3/" donde cada segmento
//            es el índice del hijo en ese nivel. Esto permite construir
//            reportes jerárquicos con subtotales por nivel (Nivel0..Nivel5).
// ─────────────────────────────────────────────
#[derive(Debug, sqlx::FromRow)]
pub struct PartidasPpto {
    /// Id único de la partida (columna 0 del DataSet / lId en GridView)
    pub id: Option<i32>,

    /// Id del presupuesto al que pertenece esta partida
    /// Solo se usa en Alta — no viene en el SELECT de lista
    pub presupuesto: Option<i32>,

    /// Ruta jerárquica del nodo: "/ppto_id/n1/n2/.../nN/"
    /// Ejemplo: "/42/1/3/2/" = presupuesto 42, rama 1, sub 3, sub 2
    /// (DrPart.GetString(0) en EscribeReng)
    pub nodo: Option<String>,

    /// Descripción de la partida — lo que aparece en el renglón del reporte
    /// (DrPart.GetString(1) en EscribeReng)
    pub concepto: Option<String>,

    /// Nombre corto de la unidad de medida (m², kg, pza, etc.)
    /// (DrPart.GetString(2) en EscribeReng)
    pub unidad_nombre: Option<String>,

    /// Id de la unidad de medida — FK a ppto_Unidades
    /// (FindControl("lUnidad").ToolTip en MttoPartidas)
    pub unidad: Option<i32>,

    /// Cantidad de unidades de esta partida
    /// Tipo SmallMoney en SQL Server → Decimal en Rust
    /// (DrPart.GetDecimal(3) en EscribeReng)
    pub cantidad: Option<Decimal>,

    /// Precio unitario de la partida
    /// Tipo Money en SQL Server → Decimal en Rust
    /// (DrPart.GetDecimal(4) en EscribeReng)
    pub precio_u: Option<Decimal>,

    /// Importe pre-calculado que el SP puede devolver (índice 5 del DataReader)
    /// En C# los reportes lo ignoraban y recalculaban en código.
    /// En Rust se expone para que el SP PostgreSQL lo calcule directamente.
    pub importe_calculado: Option<Decimal>,

    /// Tipo de cálculo del importe — determina cómo se obtiene el total:
    ///   0  = Monto fijo: total = precio_u
    ///   1  = Normal:     total = cantidad * precio_u
    ///   2  = % del padre: total = cantidad * (precio_u / 100) * monto_nivel_superior
    ///   3  = % del total: total = gran_total * (precio_u / 100)
    ///  99  = Solo título: total = 0, sin importe en el reporte
    /// (DrPart.GetInt32(6) en EscribePartida — FindControl("ddCalculo") en GridView)
    pub calculo: Option<i32>,

    /// Nivel de profundidad en el árbol jerárquico (1 = raíz, 2, 3, 4, 5...)
    /// Determina la sangría visual y los estilos CSS (Nivel0..Nivel5) en reportes
    /// (DrPart.GetInt32(7) en EscribeReng — FindControl("lNivel") en GridView)
    pub nivel: Option<i32>,
}

// ─────────────────────────────────────────────
// Método auxiliar: calcula el importe de una partida en Rust
// equivalente a EscribePartida() de los reportes C#
// ─────────────────────────────────────────────
impl PartidasPpto {
    /// Calcula el importe de la partida según su tipo de cálculo.
    ///
    /// # Parámetros
    /// - `monto_padre`: importe acumulado del nivel inmediato superior (para calculo=2)
    /// - `gran_total`:  importe total del presupuesto (para calculo=3)
    ///
    /// # Equivalente C#
    /// `EscribePartida(SqlDataReader DrPart)` en MttoPartidasRepPPTO.aspx.cs
    pub fn calcula_importe(
        &self,
        monto_padre: Decimal,
        gran_total: Decimal,
    ) -> Decimal {
        let cantidad  = self.cantidad.unwrap_or(Decimal::ZERO);
        let precio_u  = self.precio_u.unwrap_or(Decimal::ZERO);
        let calculo   = self.calculo.unwrap_or(1);
        let cien      = Decimal::from(100);

        match calculo {
            0  => precio_u,                                          // Monto fijo
            1  => cantidad * precio_u,                               // Normal
            2  => cantidad * (precio_u / cien) * monto_padre,        // % del padre
            3 if gran_total > Decimal::ZERO
               => gran_total * (precio_u / cien),                    // % del total
            99 => Decimal::ZERO,                                     // Solo título
            _  => cantidad * precio_u,                               // default = Normal
        }
    }

    /// Devuelve true si la partida es solo un título (calculo = 99)
    pub fn es_titulo(&self) -> bool {
        self.calculo == Some(99)
    }

    /// Devuelve true si la partida es de nivel raíz (nivel = 1)
    pub fn es_raiz(&self) -> bool {
        self.nivel == Some(1)
    }
}