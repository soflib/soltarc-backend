// Generación de archivos .xlsx para todos los reportes exportables.
// Cada función pública recibe &[Value] (ya serializado en el handler)
// y devuelve los bytes del archivo o un mensaje de error.

use rust_xlsxwriter::{Color, Format, Workbook};
use serde_json::Value;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn sv(v: &Value, key: &str) -> String {
    v.get(key)
        .map(|x| match x {
            Value::String(s) => s.clone(),
            Value::Null      => String::new(),
            other            => other.to_string(),
        })
        .unwrap_or_default()
}

fn nv(v: &Value, key: &str) -> f64 {
    v.get(key)
        .and_then(|x| match x {
            Value::String(s) => s.parse().ok(),
            Value::Number(n) => n.as_f64(),
            _                => None,
        })
        .unwrap_or(0.0)
}

fn hfmt() -> Format {
    Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x1E3A5F))
        .set_font_color(Color::White)
}

fn mfmt() -> Format {
    Format::new().set_num_format("#,##0.00")
}

// headers: (label, is_numeric)   keys: JSON field names (same order)
fn make_xlsx(
    sheet_name: &str,
    headers: &[(&str, bool)],
    data: &[Value],
    keys: &[&str],
) -> Result<Vec<u8>, String> {
    let mut wb = Workbook::new();
    {
        let sheet = wb.add_worksheet();
        let _ = sheet.set_name(sheet_name);
        let hf = hfmt();
        let mf = mfmt();

        for (col, (label, _)) in headers.iter().enumerate() {
            sheet
                .write_with_format(0, col as u16, *label, &hf)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, item) in data.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            for (col, (_, is_num)) in headers.iter().enumerate() {
                let key = keys[col];
                if *is_num {
                    sheet
                        .write_number_with_format(row, col as u16, nv(item, key), &mf)
                        .map_err(|e| e.to_string())?;
                } else {
                    sheet
                        .write(row, col as u16, sv(item, key))
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        sheet.autofit();
    }
    wb.save_to_buffer().map_err(|e| e.to_string())
}

// ── Reportes Financieros ─────────────────────────────────────────────────────

pub fn captura_diaria(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Captura Diaria",
        &[
            ("Tipo", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Referencia", false), ("Concepto", false), ("Monto", true),
            ("Usuario", false), ("Proyecto", false),
        ],
        data,
        &["tipo", "fecha", "banco", "cuenta", "referencia", "concepto", "monto", "usuario", "proyecto"],
    )
}

pub fn ingresos_reporte(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Ingresos",
        &[
            ("ID", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Forma Pago", false), ("Referencia", false), ("Cliente", false),
            ("Proyecto", false), ("Monto", true), ("Comentario", false),
        ],
        data,
        &["id", "fecha", "banco", "cuenta", "forma_pago", "referencia", "cliente", "proyecto", "monto", "comentario"],
    )
}

pub fn ingresos_cliente(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Ingresos por Cliente",
        &[
            ("ID", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Forma Pago", false), ("Referencia", false),
            ("Proyecto", false), ("Monto", true), ("Comentario", false),
        ],
        data,
        &["id", "fecha", "banco", "cuenta", "forma_pago", "referencia", "proyecto", "monto", "comentario"],
    )
}

pub fn egresos_centros_costo(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Egresos Centro de Costo",
        &[
            ("ID", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Forma Pago", false), ("Referencia", false), ("Proyecto", false),
            ("Proveedor", false), ("Monto", true), ("Comentario", false),
        ],
        data,
        &["id", "fecha", "banco", "cuenta", "forma_pago", "referencia", "proyecto", "proveedor", "monto", "comentario"],
    )
}

pub fn egresos_proveedor(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Egresos por Proveedor",
        &[
            ("ID", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Forma Pago", false), ("Referencia", false), ("Proyecto", false),
            ("Centro Costo", false), ("Monto", true), ("Comentario", false),
        ],
        data,
        &["id", "fecha", "banco", "cuenta", "forma_pago", "referencia", "proyecto", "centro_costo", "monto", "comentario"],
    )
}

pub fn egresos_reporte(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Egresos",
        &[
            ("ID", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Forma Pago", false), ("Referencia", false), ("Proyecto", false),
            ("Proveedor", false), ("Centro Costo", false),
            ("Monto", true), ("Comentario", false), ("Usuario", false),
        ],
        data,
        &["id", "fecha", "banco", "cuenta", "forma_pago", "referencia", "proyecto",
          "proveedor", "centro_costo", "monto", "comentario", "usuario"],
    )
}

pub fn egresos_gral(data: &[Value]) -> Result<Vec<u8>, String> {
    egresos_reporte(data)
}

// ── Reportes Proyecto ─────────────────────────────────────────────────────────

pub fn arbol_proyecto(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Arbol de Proyecto",
        &[
            ("Nodo", false), ("Nivel", false), ("Descripción", false),
            ("Estado", false), ("Proyecto", false), ("Importe", true),
        ],
        data,
        &["nodo", "nivel", "descripcion", "estado", "proyecto", "importe"],
    )
}

pub fn audita_xref(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Audita XREF",
        &[
            ("Nodo", false), ("Nivel", false), ("Descripción", false),
            ("Estado", false), ("Proyecto", false), ("Importe", true),
        ],
        data,
        &["nodo", "nivel", "descripcion", "estado", "proyecto", "importe"],
    )
}

pub fn partidas_presupuesto(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Partidas Presupuesto",
        &[
            ("Nodo", false), ("Concepto", false), ("Unidad", false),
            ("Cantidad", true), ("Precio Unit.", true), ("Importe", true),
            ("Cálculo", false), ("Nivel", false),
        ],
        data,
        &["nodo", "concepto", "unidad", "cantidad", "precio_u", "importe", "calculo", "nivel"],
    )
}

// ── Finanzas ──────────────────────────────────────────────────────────────────

pub fn flujo_caja(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Flujo de Caja",
        &[("Tipo", false), ("Banco", false), ("Monto", true)],
        data,
        &["tipo", "banco_nombre", "monto"],
    )
}

pub fn egresos_prov_proy(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Egresos Prov x Proyecto",
        &[
            ("Tipo", false), ("Proveedor", false),
            ("Proyecto", false), ("Monto", true),
        ],
        data,
        &["tipo", "proveedor", "proyecto", "monto"],
    )
}

pub fn ingresos_detalle(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Ingresos Detalle",
        &[
            ("ID", false), ("Fecha", false), ("Banco", false), ("Cuenta", false),
            ("Forma Pago", false), ("Proyecto", false), ("Monto", true),
            ("Referencia", false), ("Comentario", false),
            ("Fecha Aplica", false), ("Cliente", false),
        ],
        data,
        &["id", "fecha", "banco", "cuenta", "forma_pago", "proyecto", "monto",
          "referencia", "comentario", "fecha_aplica", "cliente"],
    )
}

// ── PPTO ──────────────────────────────────────────────────────────────────────

pub fn partidas_ppto(data: &[Value]) -> Result<Vec<u8>, String> {
    make_xlsx(
        "Partidas PPTO",
        &[
            ("Nodo", false), ("Concepto", false), ("Unidad", false),
            ("Cantidad", true), ("Precio Unit.", true), ("Importe", true),
            ("Cálculo", false), ("Nivel", false),
        ],
        data,
        &["nodo", "concepto", "unidad_nombre", "cantidad", "precio_u",
          "importe_calculado", "calculo", "nivel"],
    )
}
