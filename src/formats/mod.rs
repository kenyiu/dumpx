//! Format registry and dispatch for generated sample files.

use std::path::Path;

use ::image::ImageFormat;
use anyhow::{anyhow, Result};

use crate::naming::Tag;
use crate::template::TemplateOptions;

mod common;
mod csv;
mod image;
mod json;
mod jsonl;
mod markdown;
mod parquet;
mod pdf;
mod text;

pub const DEFAULT_FORMATS: &[&str] = &[
    "csv", "parquet", "json", "jsonl", "txt", "md", "pdf", "jpg", "png",
];

pub fn default_formats() -> Vec<String> {
    DEFAULT_FORMATS
        .iter()
        .map(|format| format.to_string())
        .collect()
}

pub fn validate_format(format: &str) -> Result<()> {
    if is_supported_format(format) {
        Ok(())
    } else {
        Err(anyhow!(
            "unsupported format: {format}; run `dumpx --list-formats`"
        ))
    }
}

pub fn is_supported_format(format: &str) -> bool {
    matches!(
        format,
        "csv" | "parquet" | "json" | "jsonl" | "txt" | "md" | "pdf" | "jpg" | "jpeg" | "png"
    )
}

pub fn extension(format: &str) -> &str {
    if format == "jpeg" {
        "jpg"
    } else {
        format
    }
}

pub fn generate_file(
    format: &str,
    target_size: u64,
    path: &Path,
    tags: &[Tag],
    templates: &TemplateOptions,
    force: bool,
) -> Result<()> {
    match format {
        "csv" => csv::write(path, target_size, tags, templates, force),
        "json" => json::write(path, target_size, tags, templates, force),
        "jsonl" => jsonl::write(path, target_size, tags, templates, force),
        "txt" => text::write(path, target_size, tags, templates, force),
        "md" => markdown::write(path, target_size, tags, templates, force),
        "pdf" => pdf::write(path, target_size, tags, templates, force),
        "jpg" | "jpeg" => image::write(path, target_size, ImageFormat::Jpeg, force),
        "png" => image::write(path, target_size, ImageFormat::Png, force),
        "parquet" => parquet::write(path, target_size, tags, force),
        _ => Err(anyhow!("unsupported format: {format}")),
    }
}
