//! Parquet file writer.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use parquet::basic::Compression;
use parquet::column::writer::ColumnWriter;
use parquet::data_type::ByteArray;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;

use crate::formats::common::{repeated_payload, rows_for_size};
use crate::naming::{tags_text, Tag};
use crate::output::create_output_file;

pub fn write(path: &Path, target_size: u64, tags: &[Tag], force: bool) -> Result<()> {
    let schema = Arc::new(parse_message_type(
        r#"
        message schema {
          REQUIRED INT64 id;
          REQUIRED BINARY tags (UTF8);
          REQUIRED BINARY payload (UTF8);
        }
        "#,
    )?);
    let props = Arc::new(
        WriterProperties::builder()
            .set_compression(Compression::UNCOMPRESSED)
            .build(),
    );
    let tag_text = tags_text(tags);

    let mut rows = rows_for_size(target_size, 180).max(1);
    loop {
        let file = create_output_file(path, force)?;
        let mut writer = SerializedFileWriter::new(file, schema.clone(), props.clone())?;
        let mut row_group = writer.next_row_group()?;

        if let Some(mut column_writer) = row_group.next_column()? {
            match column_writer.untyped() {
                ColumnWriter::Int64ColumnWriter(typed) => {
                    let values = (0..rows as i64).collect::<Vec<_>>();
                    typed.write_batch(&values, None, None)?;
                }
                _ => return Err(anyhow!("unexpected parquet id column type")),
            }
            column_writer.close()?;
        }

        if let Some(mut column_writer) = row_group.next_column()? {
            match column_writer.untyped() {
                ColumnWriter::ByteArrayColumnWriter(typed) => {
                    let tag_values = (0..rows)
                        .map(|_| ByteArray::from(tag_text.as_str()))
                        .collect::<Vec<_>>();
                    typed.write_batch(&tag_values, None, None)?;
                }
                _ => return Err(anyhow!("unexpected parquet tags column type")),
            }
            column_writer.close()?;
        }

        if let Some(mut column_writer) = row_group.next_column()? {
            match column_writer.untyped() {
                ColumnWriter::ByteArrayColumnWriter(typed) => {
                    let payloads = (0..rows)
                        .map(|row| ByteArray::from(repeated_payload(row as u64, 128).as_str()))
                        .collect::<Vec<_>>();
                    typed.write_batch(&payloads, None, None)?;
                }
                _ => return Err(anyhow!("unexpected parquet payload column type")),
            }
            column_writer.close()?;
        }

        row_group.close()?;
        writer.close()?;

        let actual_size = fs::metadata(path)?.len();
        if actual_size >= target_size {
            return Ok(());
        }
        rows = ((rows as f64) * 1.4).ceil() as usize + 1;
    }
}
