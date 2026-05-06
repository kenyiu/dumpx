//! Output reports and safe file creation helpers.

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::ValueEnum;
use serde::Serialize;

use crate::naming::Tag;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputMode {
    Text,
    Json,
    Jsonl,
}

#[derive(Debug, Serialize)]
pub struct GeneratedFile {
    pub format: String,
    pub requested_size: u64,
    pub requested_size_label: String,
    pub actual_size: u64,
    pub path: String,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize)]
pub struct RunReport {
    pub r#type: &'static str,
    pub ok: bool,
    pub out_dir: String,
    pub count: usize,
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Serialize)]
pub struct ErrorReport {
    pub r#type: &'static str,
    pub ok: bool,
    pub error: String,
}

pub fn emit_file_report(mode: OutputMode, quiet: bool, file: &GeneratedFile) -> Result<()> {
    if quiet {
        return Ok(());
    }

    match mode {
        OutputMode::Text => println!(
            "{:>7} {:>10} -> {} bytes ({})",
            file.format, file.requested_size_label, file.actual_size, file.path
        ),
        OutputMode::Json => {}
        OutputMode::Jsonl => println!(
            "{}",
            serde_json::json!({
                "type": "file",
                "ok": true,
                "file": file
            })
        ),
    }
    Ok(())
}

pub fn emit_summary_report(mode: OutputMode, report: &RunReport) -> Result<()> {
    match mode {
        OutputMode::Text => println!("generated {} file(s)", report.count),
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(report)?),
        OutputMode::Jsonl => println!("{}", serde_json::to_string(report)?),
    }
    Ok(())
}

pub fn emit_error_report(mode: OutputMode, error: &anyhow::Error) {
    match mode {
        OutputMode::Text => eprintln!("error: {error:#}"),
        OutputMode::Json => {
            let report = ErrorReport {
                r#type: "error",
                ok: false,
                error: format!("{error:#}"),
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_else(fallback_error)
            );
        }
        OutputMode::Jsonl => {
            let report = ErrorReport {
                r#type: "error",
                ok: false,
                error: format!("{error:#}"),
            };
            println!(
                "{}",
                serde_json::to_string(&report).unwrap_or_else(fallback_error)
            );
        }
    }
}

pub fn create_output_file(path: &Path, force: bool) -> Result<File> {
    reject_symlink(path)?;
    let mut options = OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    options
        .open(path)
        .with_context(|| format!("failed to create {}", path.display()))
}

pub fn write_output_bytes(path: &Path, bytes: &[u8], force: bool) -> Result<()> {
    let mut file = create_output_file(path, force)?;
    file.write_all(bytes)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn reject_symlink(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(anyhow!(
            "refusing to write through symlink: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

fn fallback_error(_: serde_json::Error) -> String {
    r#"{"type":"error","ok":false,"error":"failed to serialize error"}"#.to_string()
}
