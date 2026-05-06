//! Minimal PDF file writer.

use std::io::Write;
use std::path::Path;

use anyhow::Result;

use crate::formats::common::{pdf_escape, repeated_payload};
use crate::naming::{tags_text, Tag};
use crate::output::write_output_bytes;
use crate::template::TemplateOptions;

pub fn write(
    path: &Path,
    target_size: u64,
    tags: &[Tag],
    templates: &TemplateOptions,
    force: bool,
) -> Result<()> {
    let mut body = if let Some(header) = &templates.header {
        format!(
            "BT /F1 11 Tf 50 780 Td ({}) Tj 0 -16 Td ",
            pdf_escape(header)
        )
    } else {
        format!(
            "BT /F1 11 Tf 50 780 Td (Generated PDF sample) Tj 0 -16 Td (Tags: {}) Tj 0 -16 Td ",
            pdf_escape(&tags_text(tags))
        )
    };
    let mut row = 0_u64;
    while body.len() < target_size.saturating_sub(500) as usize {
        let line = render_pdf_line(templates, row)?;
        body.push_str(&format!("({}) Tj 0 -14 Td ", pdf_escape(&line)));
        row += 1;
    }

    loop {
        let line = render_pdf_line(templates, row)?;
        body.push_str(&format!("({}) Tj 0 -14 Td ", pdf_escape(&line)));
        row += 1;

        let pdf = build_pdf(&body)?;
        if pdf.len() as u64 >= target_size {
            write_output_bytes(path, &pdf, force)?;
            return Ok(());
        }
    }
}

fn render_pdf_line(templates: &TemplateOptions, row: u64) -> Result<String> {
    if let Some(template) = &templates.body {
        template.render(row)
    } else {
        Ok(format!("Line {row}: {}", repeated_payload(row, 72)))
    }
}

fn build_pdf(body_content: &str) -> Result<Vec<u8>> {
    let body = format!("{body_content}ET");
    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");
    let mut offsets = Vec::new();
    push_pdf_object(
        &mut pdf,
        &mut offsets,
        "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
    );
    push_pdf_object(
        &mut pdf,
        &mut offsets,
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n",
    );
    push_pdf_object(
        &mut pdf,
        &mut offsets,
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>\nendobj\n",
    );
    push_pdf_object(
        &mut pdf,
        &mut offsets,
        "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
    );

    offsets.push(pdf.len());
    write!(
        pdf,
        "5 0 obj\n<< /Length {} >>\nstream\n{}\nendstream\nendobj\n",
        body.len(),
        body
    )?;

    let xref_offset = pdf.len();
    write!(pdf, "xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1)?;
    for offset in offsets {
        writeln!(pdf, "{offset:010} 00000 n ")?;
    }
    write!(
        pdf,
        "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
    )?;
    Ok(pdf)
}

fn push_pdf_object(pdf: &mut Vec<u8>, offsets: &mut Vec<usize>, object: &str) {
    offsets.push(pdf.len());
    pdf.extend_from_slice(object.as_bytes());
}
