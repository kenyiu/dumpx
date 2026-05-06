//! Size parsing and formatting utilities.

use anyhow::{anyhow, Context, Result};

pub const DEFAULT_MAX_SIZE: u64 = 1024 * 1024 * 1024;

pub fn parse_size(input: &str) -> Result<u64> {
    let trimmed = input.trim();
    let split_at = trimmed
        .find(|ch: char| !ch.is_ascii_digit() && ch != '.')
        .unwrap_or(trimmed.len());
    let (number, unit) = trimmed.split_at(split_at);
    let number = number
        .parse::<f64>()
        .with_context(|| format!("invalid size: {input}"))?;
    if number <= 0.0 {
        return Err(anyhow!("size must be greater than zero: {input}"));
    }
    let bytes = match unit.trim() {
        "" | "B" => number,
        "b" => number / 8.0,
        "kB" | "KB" | "KiB" => number * 1_024_f64,
        "kb" | "Kb" | "Kib" => number * 1_024_f64 / 8.0,
        "MB" | "MiB" => number * 1_024_f64 * 1_024_f64,
        "Mb" | "Mib" => number * 1_024_f64 * 1_024_f64 / 8.0,
        "GB" | "GiB" => number * 1_024_f64 * 1_024_f64 * 1_024_f64,
        "Gb" | "Gib" => number * 1_024_f64 * 1_024_f64 * 1_024_f64 / 8.0,
        _ => {
            return Err(anyhow!(
                "unsupported size unit in {input}; use B, b, kB, kb, KiB, Kib, MB, Mb, MiB, Mib, GB, Gb, GiB, or Gib"
            ))
        }
    };
    if !bytes.is_finite() || bytes > u64::MAX as f64 {
        return Err(anyhow!("size is too large: {input}"));
    }
    Ok(bytes.ceil() as u64)
}

pub fn size_label(size: u64) -> String {
    if size.is_multiple_of(1024 * 1024 * 1024) {
        format!("{}GiB", size / (1024 * 1024 * 1024))
    } else if size.is_multiple_of(1024 * 1024) {
        format!("{}MiB", size / (1024 * 1024))
    } else if size.is_multiple_of(1024) {
        format!("{}KiB", size / 1024)
    } else {
        format!("{size}B")
    }
}

pub fn default_sizes() -> Vec<String> {
    vec![
        "10KiB".to_string(),
        "100KiB".to_string(),
        "1MiB".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_byte_sizes() {
        assert_eq!(parse_size("1B").unwrap(), 1);
        assert_eq!(parse_size("1b").unwrap(), 1);
        assert_eq!(parse_size("8b").unwrap(), 1);
        assert_eq!(parse_size("9b").unwrap(), 2);
        assert_eq!(parse_size("1kB").unwrap(), 1_024);
        assert_eq!(parse_size("1kb").unwrap(), 128);
        assert_eq!(parse_size("1KB").unwrap(), 1_024);
        assert_eq!(parse_size("1Kb").unwrap(), 128);
        assert_eq!(parse_size("1KiB").unwrap(), 1_024);
        assert_eq!(parse_size("1Kib").unwrap(), 128);
        assert_eq!(parse_size("1.5MiB").unwrap(), 1_572_864);
        assert_eq!(parse_size("2GiB").unwrap(), 2_147_483_648);
    }

    #[test]
    fn rejects_invalid_sizes() {
        assert!(parse_size("0KiB").is_err());
        assert!(parse_size("10tb").is_err());
        assert!(parse_size("abc").is_err());
    }
}
