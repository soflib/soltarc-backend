// Programa...: models/plan_semanal.rs
// Descripción: Modelos de dominio para el plan semanal de obra
//
// ─────────────────────────────────────────────────────────────────────────────
// POR QUÉ ESTE MODELO NO EXISTÍA EN C#
// ─────────────────────────────────────────────────────────────────────────────
// En C# oPlanSemanal.CargaPartidas() devolvía un DataSet sin tipo.
// Las columnas nunca se declararon en una entidad — vivían implícitamente
// en dos páginas ASPX que consumían el mismo DataSet:
//
//   1. AvanceSemanalPlan.aspx.cs (tablero interno — PlanDeObra)
//      Contiene el comentario más explícito de todo el proyecto:
//        // (0) [prd_Nodo].ToString()                        AS 'Nodo'
//        // (1) [prd_Nodo].GetLevel()                        AS 'Nivel'
//        // (2) [prd_Descripcion]                            AS 'Descripción'
//        // (3) [prd_FechaInicio]                            AS 'Inicia'
//        // (4) [prd_FechaFin]                               AS 'Termina'
//        // (5) DATEDIFF(wk, @FechaIni, prd_FechaInicio)     AS 'CuandoIni'
//        // (6) DATEDIFF(wk, @FechaIni, [prd_FechaFin])      AS 'CuandoFin'
//        // (7) [prd_Estado]                                  AS 'Estado'
//      Sin inferencia — es la definición exacta del SELECT del SP.
//
//   2. Cte_AvancePlanSemanal.aspx.cs (vista cliente)
//      Accede por ItemArray[0..7] con el mismo orden — confirma los índices.
//      Usa además: tRow.ItemArray[5] y [6] para calcular rango de semanas
//      en el cronograma, y tRow.ItemArray[7] para comparar con FinDeTarea.
//
// eFechas también era una clase interna en oPlanSemanal. Se define aquí
// como struct separado — ver Fechas más abajo.
// ─────────────────────────────────────────────────────────────────────────────

use time::Date;

// ─────────────────────────────────────────────
// PartidasSemanal
// Origen: sp_cpa_PlanSemPartidas(@Proyecto, @FechaIni, @Nivel)
//
// Propósito: Representa una partida del proyecto con su posición en el
//            cronograma semanal. Cada partida sabe en qué semana inicia
//            y en qué semana termina (relativas al inicio del proyecto),
//            lo que permite construir el histograma de Gantt semanal sin
//            cálculos adicionales en la capa de presentación.
//
//            La capa de presentación usa cuando_ini y cuando_fin para
//            pintar celdas coloreadas en el cronograma:
//              Rojo     (AT) = atrasado: fecha_fin < hoy y estado != fin_de_tarea
//              Verde    (ET) = en tiempo: días restantes >= 31
//              Naranja  (IP) = inicio próximo: días restantes entre 30-31
//              Amarillo (IT) = inicio inminente: días restantes entre 15-29
//              Cyan     (TE) = terminado: estado == fin_de_tarea
// ─────────────────────────────────────────────
#[derive(Debug, sqlx::FromRow)]
pub struct PartidasSemanal {
    /// Ruta jerárquica del nodo en el árbol del proyecto
    /// Equivalente a [prd_Nodo].ToString() — índice 0
    /// Ejemplo: "/42/1/3/" donde 42 es el proyecto
    pub nodo: String,

    /// Nivel de profundidad en el árbol (1=raíz, 2, 3, 4, 5...)
    /// Equivalente a [prd_Nodo].GetLevel() — índice 1
    /// Determina sangría visual y estilo CSS (Nivel0..Nivel5)
    pub nivel: i32,

    /// Descripción de la tarea/partida
    /// Equivalente a [prd_Descripcion] — índice 2
    pub descripcion: String,

    /// Fecha de inicio de la tarea
    /// Equivalente a [prd_FechaInicio] AS 'Inicia' — índice 3
    pub fecha_inicio: Option<Date>,

    /// Fecha de fin programada de la tarea
    /// Equivalente a [prd_FechaFin] AS 'Termina' — índice 4
    /// Se usa para calcular días restantes y determinar color del semáforo
    pub fecha_fin: Option<Date>,

    /// Semana de inicio relativa al inicio del proyecto (base 0)
    /// Equivalente a DATEDIFF(wk, @FechaIni, prd_FechaInicio) — índice 5
    /// Se usa para determinar en qué columna del cronograma empieza la barra
    pub cuando_ini: i32,

    /// Semana de fin relativa al inicio del proyecto (base 0)
    /// Equivalente a DATEDIFF(wk, @FechaIni, [prd_FechaFin]) — índice 6
    /// Se usa para determinar en qué columna del cronograma termina la barra
    pub cuando_fin: i32,

    /// Id del estado de la tarea (FK a cpa_Catalogos tipo 1)
    /// Equivalente a [prd_Estado] — índice 7
    /// Se compara con fin_de_tarea (configuración del sistema) para
    /// determinar si la tarea está completada (color Cyan)
    pub estado: i32,
}

// ─────────────────────────────────────────────
// Método auxiliar: determina el color del semáforo de avance
// Equivalente a la lógica de LlenaTabla() / LlenaTablaExcel()
// en AvanceSemanalPlan.aspx.cs y Cte_AvancePlanSemanal.aspx.cs
// ─────────────────────────────────────────────
impl PartidasSemanal {
    /// Devuelve el código de semáforo de avance para una semana dada.
    ///
    /// # Parámetros
    /// - `semana_actual`: semana actual del proyecto (días desde inicio / 7)
    /// - `fin_de_tarea`:  id de estado que indica tarea terminada (de configuración)
    /// - `hoy`:           fecha de hoy para calcular días restantes
    ///
    /// # Retorna
    /// - `Some(SemaforoPlan)` si la semana cae dentro del rango de la tarea
    /// - `None` si la semana está fuera del rango (celda vacía/blanca)
    ///
    /// # Equivalente C#
    /// El bloque `if (co >= SemIni && co <= SemFin)` en LlenaTabla()
    pub fn semaforo(
        &self,
        semana: i32,
        semana_actual: i32,
        fin_de_tarea: i32,
        hoy: Date,
    ) -> Option<SemaforoPlan> {
        let sem_ini = self.cuando_ini + 1;
        let sem_fin = self.cuando_fin;

        if semana < sem_ini || semana > sem_fin {
            return None; // fuera del rango — celda blanca
        }

        let dias_restantes = self.fecha_fin.map(|f| {
            (f - hoy).whole_days()
        }).unwrap_or(0);

        let semaforo = if dias_restantes < 0 && self.estado != fin_de_tarea {
            SemaforoPlan::Atrasado        // Rojo  — AT
        } else if self.estado == fin_de_tarea {
            SemaforoPlan::Terminado       // Cyan  — TE
        } else if dias_restantes >= 31 {
            SemaforoPlan::EnTiempo        // Verde — ET
        } else if dias_restantes >= 30 {
            SemaforoPlan::InicioProximo   // Naranja — IP
        } else {
            SemaforoPlan::InicioInminente // Amarillo — IT
        };

        Some(semaforo)
    }
}

// ─────────────────────────────────────────────
// SemaforoPlan
// Los cinco estados de avance del cronograma semanal.
// Equivalente a los códigos "AT"/"ET"/"IP"/"IT"/"TE"
// usados en LlenaTablaExcel() para colorear celdas Excel.
// ─────────────────────────────────────────────
#[derive(Debug, PartialEq)]
pub enum SemaforoPlan {
    /// Atrasado — fecha_fin pasó y tarea no terminada (Rojo / "AT")
    Atrasado,
    /// En tiempo — más de 31 días para terminar (Verde / "ET")
    EnTiempo,
    /// Inicio próximo — entre 30 y 31 días para terminar (Naranja / "IP")
    InicioProximo,
    /// Inicio inminente — entre 15 y 29 días para terminar (Amarillo / "IT")
    InicioInminente,
    /// Terminado — estado == fin_de_tarea (Cyan / "TE")
    Terminado,
}

impl SemaforoPlan {
    /// Devuelve el código de dos letras usado en exportación Excel
    /// Equivalente a los literales "AT", "ET", "IP", "IT", "TE"
    pub fn codigo(&self) -> &'static str {
        match self {
            SemaforoPlan::Atrasado        => "AT",
            SemaforoPlan::EnTiempo        => "ET",
            SemaforoPlan::InicioProximo   => "IP",
            SemaforoPlan::InicioInminente => "IT",
            SemaforoPlan::Terminado       => "TE",
        }
    }
}

// ─────────────────────────────────────────────
// Fechas
// Origen: sp_cpa_PlanSemFechas(@Proyecto)
//
// Propósito: Devuelve el rango temporal del plan del proyecto —
//            fecha de inicio, fecha de fin y número total de semanas.
//            Con estos tres valores la capa de presentación construye
//            los encabezados del cronograma (S1, S2, ... SN) y calcula
//            en qué semana relativa se encuentra hoy.
//
// En C# era la clase interna oPlanSemanal.eFechas.
// Se define aquí como struct independiente para ser retornado
// por plan_semanal::fechas() en el DAL.
// ─────────────────────────────────────────────
#[derive(Debug, sqlx::FromRow)]
pub struct Fechas {
    /// Fecha de inicio del plan del proyecto
    /// Equivalente a DrFechas.GetDateTime(0) en oPlanSemanal.Fechas()
    pub fecha_ini: Date,

    /// Fecha de fin del plan del proyecto
    /// Equivalente a DrFechas.GetDateTime(1)
    pub fecha_fin: Date,

    /// Número total de semanas del proyecto (duración del cronograma)
    /// Equivalente a DrFechas.GetInt32(2)
    /// Es el número de columnas del histograma semanal
    pub num_semanas: i32,
}

impl Fechas {
    /// Calcula la semana actual relativa al inicio del proyecto.
    /// Equivalente al cálculo en CargaAvance():
    ///   FP.SemanaActual = DateTime.Today.Subtract(FP.FechaInicial).Days / 7
    pub fn semana_actual(&self, hoy: Date) -> i32 {
        let dias = (hoy - self.fecha_ini).whole_days();
        (dias / 7) as i32
    }

    /// Devuelve true si el proyecto está atrasado (hoy > fecha_fin)
    pub fn esta_atrasado(&self, hoy: Date) -> bool {
        hoy > self.fecha_fin
    }

    /// Días de atraso si el proyecto ya pasó su fecha fin (0 si está en tiempo)
    pub fn dias_atraso(&self, hoy: Date) -> i64 {
        if hoy > self.fecha_fin {
            (hoy - self.fecha_fin).whole_days()
        } else {
            0
        }
    }
}