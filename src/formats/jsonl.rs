//! JSON Lines file writer.

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
        let mut row = 1_u64;
        while writer.stream_position()? < target_size {
            writeln!(writer, "{}", template.render(row)?)?;
            row += 1;
        }
        return Ok(());
    }

    let tags_json = tags_json(tags)?;
    let mut row = 0_u64;
    while writer.stream_position()? < target_size {
        writeln!(
            writer,
            "{{\"id\":{row},\"name\":\"User {row}\",\"active\":{},\"tags\":{},\"payload\":\"{}\"}}",
            row.is_multiple_of(2),
            tags_json,
            repeated_payload(row, 128)
        )?;
        row += 1;
    }
    Ok(())
}
