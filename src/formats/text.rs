//! Plain text file writer.

use std::io::{BufWriter, Seek, Write};
use std::path::Path;

use anyhow::Result;

use crate::formats::common::repeated_payload;
use crate::naming::{tags_text, Tag};
use crate::output::create_output_file;
use crate::template::TemplateOptions;

pub fn write(
    path: &Path,
    target_size: u64,
    tags: &[Tag],
    templates: &TemplateOptions,
    force: bool,
) -> Result<()> {
    let mut writer = BufWriter::new(create_output_file(path, force)?);
    if let Some(template) = &templates.body {
        if let Some(header) = &templates.header {
            writeln!(writer, "{header}")?;
        }
        let mut row = 1_u64;
        while writer.stream_position()? < target_size {
            writeln!(writer, "{}", template.render(row)?)?;
            row += 1;
        }
        return Ok(());
    }

    writeln!(writer, "Tags: {}", tags_text(tags))?;
    let mut row = 0_u64;
    while writer.stream_position()? < target_size {
        writeln!(
            writer,
            "Line {row}: The quick brown fox jumps over generated payload {}.",
            repeated_payload(row, 128)
        )?;
        row += 1;
    }
    Ok(())
}
