//! JSON array file writer.

use std::io::{BufWriter, Seek, Write};
use std::path::Path;

use anyhow::Result;

use crate::formats::common::repeated_payload;
use crate::naming::{tags_json, Tag};
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
        writer.write_all(b"[\n")?;
        let mut row = 1_u64;
        while writer.stream_position()? + 4 < target_size {
            if row > 1 {
                writer.write_all(b",\n")?;
            }
            write!(writer, "  {}", template.render(row)?)?;
            row += 1;
        }
        writer.write_all(b"\n]\n")?;
        return Ok(());
    }

    let tags_json = tags_json(tags)?;
    writer.write_all(b"[\n")?;
    let mut row = 0_u64;
    while writer.stream_position()? + 4 < target_size {
        if row > 0 {
            writer.write_all(b",\n")?;
        }
        write!(
            writer,
            "  {{\"id\":{row},\"name\":\"User {row}\",\"active\":{},\"tags\":{},\"payload\":\"{}\"}}",
            row.is_multiple_of(2),
            tags_json,
            repeated_payload(row, 128)
        )?;
        row += 1;
    }
    writer.write_all(b"\n]\n")?;
    Ok(())
}
