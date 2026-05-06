//! Command-line parsing and orchestration for `dumpx`.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

use crate::formats::{
    default_formats, extension, generate_file, is_supported_format, validate_format,
    DEFAULT_FORMATS,
};
use crate::naming::{file_stem_prefix, parse_tags};
use crate::output::{
    emit_error_report, emit_file_report, emit_summary_report, GeneratedFile, OutputMode, RunReport,
};
use crate::size::{default_sizes, parse_size, size_label, DEFAULT_MAX_SIZE};
use crate::template::{Template, TemplateOptions};

const DEFAULT_MAX_FILES: usize = 100;

#[derive(Parser, Debug)]
#[command(
    name = "dumpx",
    version,
    about = "Generate deterministic sample files in common formats and sizes",
    long_about = "dumpx is a beta CLI. Use with caution. It generates local sample files for ingestion, upload, parser, and storage tests. It can create CSV, Parquet, JSON, JSONL, TXT, Markdown, PDF, JPG, and PNG files at one or more target sizes.",
    after_help = "Examples:
  dumpx --size 10KiB --format csv,jsonl --tag suite=smoke
  dumpx csv,json 100MB
  dumpx csv 10MB .
  dumpx 100MB csv
  dumpx --out-dir fixtures --size 1MiB,10MiB --format parquet,png --output json
  dumpx --json -s 100KiB -f txt -t run=ci

For agents, prefer:
  dumpx --json -s 100KiB -f txt -t run=ci

Readable equivalent:
  dumpx --quiet --output json --size 100KiB --format txt --tag run=ci

By default dumpx refuses to overwrite files and caps each file at 1GiB and each run at 100 files. Use --force and --allow-large when you intentionally need those behaviors.

Beta release: use with caution.

Sizes are case-sensitive for bytes vs bits: B=bytes and b=bits. All K/M/G prefixes use powers of 1024, so kB, KB, and KiB are equivalent; kb, Kb, and Kib are equivalent bit units."
)]
struct Args {
    /// Output directory for generated files.
    #[arg(short, long, default_value = ".")]
    out_dir: PathBuf,

    /// Target sizes. Accepts repeated values or comma-separated values.
    #[arg(short, long = "size", visible_alias = "sizes", value_delimiter = ',')]
    sizes: Option<Vec<String>>,

    /// File formats. Accepts repeated values or comma-separated values.
    #[arg(
        short,
        long = "format",
        visible_alias = "formats",
        value_delimiter = ','
    )]
    formats: Option<Vec<String>>,

    /// Metadata tag to attach to generated content, filenames, and reports. Repeatable KEY=VALUE.
    #[arg(short = 't', long = "tag", value_name = "KEY=VALUE")]
    tags: Vec<String>,

    /// Prefix used in generated file names.
    #[arg(long, default_value = "sample")]
    prefix: String,

    /// Row/item template for csv, json, jsonl, txt, md, and pdf.
    #[arg(long)]
    template: Option<String>,

    /// Read the row/item template from a file.
    #[arg(long)]
    template_file: Option<PathBuf>,

    /// Optional header written once before templated rows.
    #[arg(long)]
    template_header: Option<String>,

    /// Overwrite existing output files. Symlink destinations are still refused.
    #[arg(long)]
    force: bool,

    /// Allow file sizes above the default 1GiB safety limit.
    #[arg(long)]
    allow_large: bool,

    /// Maximum number of files to generate in one run.
    #[arg(long, default_value_t = DEFAULT_MAX_FILES)]
    max_files: usize,

    /// Output style written to stdout.
    #[arg(long = "output", visible_alias = "report", value_enum, default_value_t = OutputMode::Text)]
    output: OutputMode,

    /// Suppress per-file progress/events. With --output json, stdout is one summary object.
    #[arg(short, long)]
    quiet: bool,

    /// Compact shorthand for --quiet --output json.
    #[arg(long)]
    json: bool,

    /// Deprecated shorthand for --quiet --output json.
    #[arg(long)]
    agent: bool,

    /// Print supported formats and exit.
    #[arg(long)]
    list_formats: bool,

    /// Positional format, size, and output directory tokens. Examples: `csv 100MB`, `csv,json 100MB`, or `csv 10MB .`.
    #[arg(value_name = "FORMAT_SIZE_OR_DIR")]
    items: Vec<String>,
}

impl Args {
    fn output_mode(&self) -> OutputMode {
        if self.json || self.agent {
            OutputMode::Json
        } else {
            self.output
        }
    }

    fn quiet(&self) -> bool {
        self.quiet || self.json || self.agent
    }
}

pub fn run_from_env() -> ExitCode {
    let args = match parse_args_from_env() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("error: {error:#}");
            return ExitCode::FAILURE;
        }
    };

    let output_mode = args.output_mode();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            emit_error_report(output_mode, &error);
            ExitCode::FAILURE
        }
    }
}

fn parse_args_from_env() -> Result<Args> {
    if env::args_os().len() == 1 {
        prompt_args()
    } else {
        Ok(Args::parse())
    }
}

fn run(args: Args) -> Result<()> {
    if args.list_formats {
        for format in DEFAULT_FORMATS {
            println!("{format}");
        }
        return Ok(());
    }

    let tags = parse_tags(&args.tags)?;
    let (positional_formats, positional_sizes, positional_out_dir) =
        infer_positional_items(&args.items)?;
    let out_dir = positional_out_dir.unwrap_or_else(|| args.out_dir.clone());
    let output_mode = args.output_mode();
    let quiet = args.quiet();
    let templates = load_templates(&args)?;
    let size_inputs = merge_or_default(args.sizes, positional_sizes, default_sizes());
    let sizes = size_inputs
        .iter()
        .map(|size| parse_size(size))
        .collect::<Result<Vec<_>>>()?;
    let formats = merge_or_default(args.formats, positional_formats, default_formats());
    validate_generation_limits(&sizes, formats.len(), args.allow_large, args.max_files)?;
    let file_stem_prefix = file_stem_prefix(&args.prefix, &tags)?;

    fs::create_dir_all(&out_dir)
        .with_context(|| format!("failed to create {}", out_dir.display()))?;

    let mut generated_files = Vec::new();
    for format in formats {
        let format = format.to_ascii_lowercase();
        validate_format(&format)?;

        for target_size in &sizes {
            let file_name = format!(
                "{}_{}.{}",
                file_stem_prefix,
                size_label(*target_size),
                extension(&format)
            );
            let path = out_dir.join(file_name);
            generate_file(&format, *target_size, &path, &tags, &templates, args.force)?;
            let actual_size = fs::metadata(&path)?.len();
            let generated = GeneratedFile {
                format: format.clone(),
                requested_size: *target_size,
                requested_size_label: size_label(*target_size),
                actual_size,
                path: path.display().to_string(),
                tags: tags.clone(),
            };
            emit_file_report(output_mode, quiet, &generated)?;
            generated_files.push(generated);
        }
    }

    let report = RunReport {
        r#type: "summary",
        ok: true,
        out_dir: out_dir.display().to_string(),
        count: generated_files.len(),
        files: generated_files,
    };
    emit_summary_report(output_mode, &report)?;
    Ok(())
}

fn prompt_args() -> Result<Args> {
    eprintln!("No arguments provided. Enter values below, or press Enter for defaults.");
    let out_dir = prompt_default("Output directory", ".")?;
    let sizes = split_prompt_values(&prompt_default("Sizes", "10KiB,100KiB,1MiB")?);
    let formats_input = prompt_default("Formats", "csv,parquet,json,jsonl,txt,md,pdf,jpg,png")?;
    let formats = split_prompt_values(&formats_input);
    let tags = split_prompt_values(&prompt_default("Tags KEY=VALUE comma-separated", "")?);
    let prefix = prompt_default("Filename prefix", "sample")?;
    let output = parse_output_mode(&prompt_default("Output text|json|jsonl", "text")?)?;

    Ok(Args {
        out_dir: PathBuf::from(out_dir),
        sizes: Some(sizes),
        formats: Some(formats),
        tags,
        prefix,
        template: None,
        template_file: None,
        template_header: None,
        force: false,
        allow_large: false,
        max_files: DEFAULT_MAX_FILES,
        output,
        quiet: false,
        json: false,
        agent: false,
        list_formats: false,
        items: Vec::new(),
    })
}

fn prompt_default(label: &str, default: &str) -> Result<String> {
    if default.is_empty() {
        eprint!("{label}: ");
    } else {
        eprint!("{label} [{default}]: ");
    }
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let value = input.trim();
    if value.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(value.to_string())
    }
}

fn split_prompt_values(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_output_mode(input: &str) -> Result<OutputMode> {
    match input.trim().to_ascii_lowercase().as_str() {
        "text" => Ok(OutputMode::Text),
        "json" => Ok(OutputMode::Json),
        "jsonl" => Ok(OutputMode::Jsonl),
        _ => Err(anyhow!(
            "unsupported output mode `{input}`; use text, json, or jsonl"
        )),
    }
}

fn infer_positional_items(items: &[String]) -> Result<(Vec<String>, Vec<String>, Option<PathBuf>)> {
    let mut formats = Vec::new();
    let mut sizes = Vec::new();
    let mut out_dir = None;

    for item in items {
        let parts = item
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        let is_group = parts.len() > 1;

        for part in parts {
            let normalized_format = part.to_ascii_lowercase();
            if is_supported_format(&normalized_format) {
                formats.push(normalized_format);
            } else if parse_size(part).is_ok() {
                sizes.push(part.to_string());
            } else if !is_group && out_dir.is_none() && !formats.is_empty() && !sizes.is_empty() {
                out_dir = Some(PathBuf::from(part));
            } else {
                return Err(anyhow!(
                    "could not infer positional argument `{part}` as a format, size, or output directory"
                ));
            }
        }
    }

    Ok((formats, sizes, out_dir))
}

fn merge_or_default(
    flagged: Option<Vec<String>>,
    positional: Vec<String>,
    default_values: Vec<String>,
) -> Vec<String> {
    let mut values = flagged.unwrap_or_default();
    values.extend(positional);
    if values.is_empty() {
        default_values
    } else {
        values
    }
}

fn validate_generation_limits(
    sizes: &[u64],
    format_count: usize,
    allow_large: bool,
    max_files: usize,
) -> Result<()> {
    if max_files == 0 {
        return Err(anyhow!("--max-files must be greater than zero"));
    }
    let file_count = sizes
        .len()
        .checked_mul(format_count)
        .ok_or_else(|| anyhow!("requested file count overflowed"))?;
    if file_count > max_files {
        return Err(anyhow!(
            "requested {file_count} files, which exceeds --max-files {max_files}"
        ));
    }
    if !allow_large {
        for size in sizes {
            if *size > DEFAULT_MAX_SIZE {
                return Err(anyhow!(
                    "requested size {} exceeds default limit {}; pass --allow-large to override",
                    size_label(*size),
                    size_label(DEFAULT_MAX_SIZE)
                ));
            }
        }
    }
    Ok(())
}

fn load_templates(args: &Args) -> Result<TemplateOptions> {
    if args.template.is_some() && args.template_file.is_some() {
        return Err(anyhow!(
            "use either --template or --template-file, not both"
        ));
    }

    let body = if let Some(template) = &args.template {
        Some(Template::new(template.clone())?)
    } else if let Some(path) = &args.template_file {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read template file {}", path.display()))?;
        Some(Template::new(contents)?)
    } else {
        None
    };

    Ok(TemplateOptions {
        body,
        header: args.template_header.clone(),
    })
}
