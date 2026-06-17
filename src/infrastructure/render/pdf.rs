// Generación de archivos PDF para los reportes que lo requieren:
//   - Recibo de Honorarios
//   - Presupuesto
//   - Avance de Obra
//
// API: printpdf 0.9.x
//   PdfDocument::new(name) + PdfPage::new(Mm, Mm, Vec<Op>)
//   Op::StartTextSection / SetFont / SetTextCursor / ShowText / EndTextSection
//   Op::DrawLine { line: Line { points: Vec<LinePoint>, is_closed: bool } }

use printpdf::{
    BuiltinFont, Line, LinePoint, Mm, Op, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Pt, RawImage, TextItem, XObjectTransform,
};
use serde_json::Value;

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn text(ops: &mut Vec<Op>, txt: &str, size: f32, x: f32, y: f32, font: BuiltinFont) {
    if txt.is_empty() { return; }
    ops.push(Op::StartTextSection);
    ops.push(Op::SetFont {
        font: PdfFontHandle::Builtin(font),
        size: Pt(size),
    });
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x), Mm(y)),
    });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(txt.to_string())],
    });
    ops.push(Op::EndTextSection);
}

fn hline(ops: &mut Vec<Op>, y: f32) {
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint { p: Point::new(Mm(15.0), Mm(y)), bezier: false },
                LinePoint { p: Point::new(Mm(195.0), Mm(y)), bezier: false },
            ],
            is_closed: false,
        },
    });
}

fn new_page(ops: Vec<Op>) -> PdfPage {
    PdfPage::new(Mm(210.0), Mm(297.0), ops)
}

/// Inserta el logo del tenant en la esquina superior derecha de la primera página.
/// Best-effort: si no hay logo o no se decodifica, el PDF sale sin él.
fn draw_logo(doc: &mut PdfDocument, ops: &mut Vec<Op>, logo: Option<&[u8]>) {
    let bytes = match logo {
        Some(b) if !b.is_empty() => b,
        _ => return,
    };
    let mut warnings = Vec::new();
    let img = match RawImage::decode_from_bytes(bytes, &mut warnings) {
        Ok(i) => i,
        Err(_) => return,
    };
    let w_px = img.width as f32;
    let h_px = img.height as f32;
    if w_px <= 0.0 || h_px <= 0.0 { return; }
    let id = doc.add_image(&img);

    // Tamaño objetivo: alto 14mm, ancho máx 50mm, conservando proporción.
    let mm = 2.834_65_f32; // mm → pt
    let mut disp_h = 14.0 * mm;
    let mut disp_w = disp_h * (w_px / h_px);
    let max_w = 50.0 * mm;
    if disp_w > max_w {
        disp_w = max_w;
        disp_h = disp_w * (h_px / w_px);
    }

    // Esquina superior derecha (margen derecho 195mm; página de 297mm de alto).
    let right = 195.0 * mm;
    let top   = 290.0 * mm;
    let scale = disp_w / w_px; // con dpi=72, 1px = 1pt

    ops.push(Op::UseXobject {
        id,
        transform: XObjectTransform {
            translate_x: Some(Pt(right - disp_w)),
            translate_y: Some(Pt(top - disp_h)),
            rotate: None,
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(72.0),
        },
    });
}

// ── Recibo de Honorarios ──────────────────────────────────────────────────────

pub fn recibo_honorarios(data: &Value, logo: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new("Recibo de Honorarios");
    let mut ops: Vec<Op> = Vec::new();
    draw_logo(&mut doc, &mut ops, logo);

    let bold = BuiltinFont::HelveticaBold;
    let reg  = BuiltinFont::Helvetica;

    let cte_prov    = sv(data, "cte_prov");
    let rfc_curp    = sv(data, "rfc_curp");
    let proyecto    = sv(data, "proyecto");
    let forma_pago  = sv(data, "forma_pago");
    let fecha       = sv(data, "fecha_aplica");
    let referencia  = sv(data, "referencia");
    let comentarios = sv(data, "comentarios");
    let monto       = nv(data, "monto");
    let banco       = sv(data, "banco");

    text(&mut ops, "RECIBO DE HONORARIOS", 18.0, 15.0, 270.0, bold);
    hline(&mut ops, 265.0);

    let mut y = 255.0_f32;
    let paso = 12.0_f32;
    let campos: &[(&str, &str)] = &[
        ("Nombre / Razon Social:", &cte_prov),
        ("RFC / CURP:",            &rfc_curp),
        ("Proyecto:",              &proyecto),
        ("Banco:",                 &banco),
        ("Forma de Pago:",         &forma_pago),
        ("Fecha:",                 &fecha),
        ("Referencia:",            &referencia),
        ("Concepto:",              &comentarios),
    ];

    for (label, valor) in campos {
        text(&mut ops, label, 10.0, 15.0, y, bold);
        text(&mut ops, valor, 10.0, 65.0, y, reg);
        y -= paso;
    }

    hline(&mut ops, y - 4.0);
    y -= 14.0;
    text(&mut ops, "IMPORTE:", 13.0, 15.0, y, bold);
    text(&mut ops, &format!("$ {:>12.2}", monto), 13.0, 65.0, y, bold);

    y -= 40.0;
    text(&mut ops, "____________________________", 10.0, 40.0, y, reg);
    text(&mut ops, "Firma", 10.0, 70.0, y - 6.0, reg);

    doc.pages.push(new_page(ops));
    Ok(doc.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

// ── Presupuesto PDF ───────────────────────────────────────────────────────────

pub fn presupuesto(presupuesto_id: i32, partidas: &[Value], logo: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let mut doc  = PdfDocument::new("Presupuesto");
    let mut ops: Vec<Op> = Vec::new();
    draw_logo(&mut doc, &mut ops, logo);
    let mut y    = 275.0_f32;

    let bold = BuiltinFont::HelveticaBold;
    let reg  = BuiltinFont::Helvetica;
    let mono = BuiltinFont::Courier;

    // Encabezado
    text(&mut ops, &format!("PRESUPUESTO  #{}", presupuesto_id), 14.0, 15.0, y, bold);
    y -= 8.0;
    hline(&mut ops, y);
    y -= 10.0;

    // Cabecera de columnas
    text(&mut ops, "Nodo",     8.0, 15.0,  y, bold);
    text(&mut ops, "Concepto", 8.0, 38.0,  y, bold);
    text(&mut ops, "Unidad",   8.0, 115.0, y, bold);
    text(&mut ops, "Cantidad", 8.0, 135.0, y, bold);
    text(&mut ops, "P.Unit.",  8.0, 155.0, y, bold);
    text(&mut ops, "Importe",  8.0, 172.0, y, bold);
    y -= 6.0;
    hline(&mut ops, y);
    y -= 8.0;

    let mut total = 0.0_f64;

    for p in partidas {
        if y < 25.0 {
            doc.pages.push(new_page(ops));
            ops = Vec::new();
            y = 275.0;
        }

        let nodo    = sv(p, "nodo");
        let concepto = sv(p, "concepto");
        let unidad  = sv(p, "unidad");
        let cant    = nv(p, "cantidad");
        let pu      = nv(p, "precio_u");
        let imp     = nv(p, "importe");
        let nivel   = nv(p, "nivel") as i32;

        total += imp;

        let indent = 38.0 + (nivel.max(1) - 1) as f32 * 4.0;
        let row_font = if nivel <= 1 { bold } else { reg };

        text(&mut ops, &nodo,                          8.0, 15.0,   y, mono);
        text(&mut ops, &concepto,                      8.0, indent, y, row_font);
        text(&mut ops, &unidad,                        8.0, 115.0,  y, reg);
        text(&mut ops, &format!("{:>8.2}", cant),      8.0, 135.0,  y, reg);
        text(&mut ops, &format!("{:>10.2}", pu),       8.0, 152.0,  y, reg);
        text(&mut ops, &format!("{:>12.2}", imp),      8.0, 170.0,  y, reg);
        y -= 7.0;
    }

    y -= 4.0;
    hline(&mut ops, y);
    y -= 8.0;
    text(&mut ops, "TOTAL:", 10.0, 140.0, y, bold);
    text(&mut ops, &format!("{:>14.2}", total), 10.0, 160.0, y, bold);

    doc.pages.push(new_page(ops));
    Ok(doc.save(&PdfSaveOptions::default(), &mut Vec::new()))
}

// ── Avance de Obra PDF ────────────────────────────────────────────────────────

pub fn avance_obra(
    proyecto_id: i32,
    ingresos: &[Value],
    egresos: &[Value],
    logo: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let mut doc = PdfDocument::new("Avance de Obra");
    let mut ops: Vec<Op> = Vec::new();
    draw_logo(&mut doc, &mut ops, logo);
    let mut y   = 275.0_f32;

    let bold = BuiltinFont::HelveticaBold;
    let reg  = BuiltinFont::Helvetica;

    // Título
    text(&mut ops, &format!("AVANCE DE OBRA  - Proyecto #{}", proyecto_id), 14.0, 15.0, y, bold);
    y -= 8.0;
    hline(&mut ops, y);
    y -= 14.0;

    // ── Ingresos ──
    let total_ing: f64 = ingresos.iter().map(|r| nv(r, "monto")).sum();

    text(&mut ops, "INGRESOS", 11.0, 15.0, y, bold);
    y -= 8.0;
    text(&mut ops, "Fecha",      8.0, 15.0,  y, bold);
    text(&mut ops, "Concepto",   8.0, 45.0,  y, bold);
    text(&mut ops, "Referencia", 8.0, 120.0, y, bold);
    text(&mut ops, "Monto",      8.0, 165.0, y, bold);
    y -= 6.0;
    hline(&mut ops, y);
    y -= 7.0;

    for r in ingresos {
        if y < 25.0 {
            doc.pages.push(new_page(ops));
            ops = Vec::new();
            y = 275.0;
        }
        text(&mut ops, &sv(r, "fecha"),      8.0, 15.0,  y, reg);
        text(&mut ops, &sv(r, "concepto"),   8.0, 45.0,  y, reg);
        text(&mut ops, &sv(r, "referencia"), 8.0, 120.0, y, reg);
        text(&mut ops, &format!("{:>12.2}", nv(r, "monto")), 8.0, 162.0, y, reg);
        y -= 6.0;
    }
    y -= 2.0;
    hline(&mut ops, y);
    y -= 8.0;
    text(&mut ops, "Total Ingresos:", 9.0, 120.0, y, bold);
    text(&mut ops, &format!("{:>12.2}", total_ing), 9.0, 162.0, y, bold);

    // ── Egresos ──
    let total_egr: f64 = egresos.iter().map(|r| nv(r, "monto")).sum();

    y -= 16.0;
    if y < 60.0 {
        doc.pages.push(new_page(ops));
        ops = Vec::new();
        y = 275.0;
    }

    text(&mut ops, "EGRESOS", 11.0, 15.0, y, bold);
    y -= 8.0;
    text(&mut ops, "Fecha",      8.0, 15.0,  y, bold);
    text(&mut ops, "Concepto",   8.0, 45.0,  y, bold);
    text(&mut ops, "Referencia", 8.0, 120.0, y, bold);
    text(&mut ops, "Monto",      8.0, 165.0, y, bold);
    y -= 6.0;
    hline(&mut ops, y);
    y -= 7.0;

    for r in egresos {
        if y < 25.0 {
            doc.pages.push(new_page(ops));
            ops = Vec::new();
            y = 275.0;
        }
        text(&mut ops, &sv(r, "fecha"),      8.0, 15.0,  y, reg);
        text(&mut ops, &sv(r, "concepto"),   8.0, 45.0,  y, reg);
        text(&mut ops, &sv(r, "referencia"), 8.0, 120.0, y, reg);
        text(&mut ops, &format!("{:>12.2}", nv(r, "monto")), 8.0, 162.0, y, reg);
        y -= 6.0;
    }
    y -= 2.0;
    hline(&mut ops, y);
    y -= 8.0;
    text(&mut ops, "Total Egresos:", 9.0, 120.0, y, bold);
    text(&mut ops, &format!("{:>12.2}", total_egr), 9.0, 162.0, y, bold);

    // ── Saldo ──
    y -= 14.0;
    let saldo = total_ing - total_egr;
    hline(&mut ops, y + 4.0);
    text(&mut ops, "SALDO:", 10.0, 120.0, y, bold);
    text(&mut ops, &format!("{:>12.2}", saldo), 10.0, 162.0, y, bold);

    doc.pages.push(new_page(ops));
    Ok(doc.save(&PdfSaveOptions::default(), &mut Vec::new()))
}
