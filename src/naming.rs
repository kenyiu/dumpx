//! File naming and user-provided metadata helpers.

use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

pub fn parse_tags(inputs: &[String]) -> Result<Vec<Tag>> {
    inputs.iter().map(|input| parse_tag(input)).collect()
}

pub fn parse_tag(input: &str) -> Result<Tag> {
    let (key, value) = input
        .split_once('=')
        .ok_or_else(|| anyhow!("invalid tag `{input}`; expected KEY=VALUE"))?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return Err(anyhow!(
            "invalid tag `{input}`; key and value must be non-empty"
        ));
    }
    if !key
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(anyhow!(
            "invalid tag key `{key}`; use letters, numbers, _, -, or ."
        ));
    }
    Ok(Tag {
        key: key.to_string(),
        value: value.to_string(),
    })
}

pub fn file_stem_prefix(prefix: &str, tags: &[Tag]) -> Result<String> {
    let prefix = sanitize_file_part(prefix);
    if prefix.is_empty() {
        return Err(anyhow!(
            "prefix must contain at least one filename-safe character"
        ));
    }

    let mut parts = vec![prefix];
    for tag in tags {
        parts.push(format!(
            "{}-{}",
            sanitize_file_part(&tag.key),
            sanitize_file_part(&tag.value)
        ));
    }
    Ok(parts.join("_"))
}

pub fn generated_file_name(
    template: Option<&str>,
    prefix: &str,
    format: &str,
    size: &str,
    extension: &str,
    index: usize,
) -> Result<String> {
    let file_name = if let Some(template) = template {
        template
            .replace("{prefix}", prefix)
            .replace("{format}", format)
            .replace("{size}", size)
            .replace("{extension}", extension)
            .replace("{index}", &index.to_string())
    } else {
        format!("{prefix}_{size}.{extension}")
    };

    validate_file_name(&file_name)?;
    Ok(file_name)
}

pub fn tags_text(tags: &[Tag]) -> String {
    if tags.is_empty() {
        return "none".to_string();
    }
    tags.iter()
        .map(|tag| format!("{}={}", tag.key, tag.value))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn tags_json(tags: &[Tag]) -> Result<String> {
    Ok(serde_json::to_string(tags)?)
}

fn validate_file_name(file_name: &str) -> Result<()> {
    if file_name.trim().is_empty() {
        return Err(anyhow!("custom file name must not be empty"));
    }
    if file_name == "." || file_name == ".." {
        return Err(anyhow!("custom file name must name a file"));
    }
    if file_name.contains('/') || file_name.contains('\\') {
        return Err(anyhow!("custom file name must not contain path separators"));
    }
    Ok(())
}

fn sanitize_file_part(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tags() {
        let tags = parse_tags(&["suite=smoke".to_string(), "owner=agent".to_string()]).unwrap();
        assert_eq!(tags[0].key, "suite");
        assert_eq!(tags[0].value, "smoke");
        assert_eq!(tags_text(&tags), "suite=smoke,owner=agent");
    }

    #[test]
    fn rejects_invalid_tags() {
        assert!(parse_tag("suite").is_err());
        assert!(parse_tag("=smoke").is_err());
        assert!(parse_tag("suite=").is_err());
        assert!(parse_tag("bad key=value").is_err());
    }

    #[test]
    fn builds_tagged_file_stem_prefix() {
        let tags =
            parse_tags(&["suite=smoke test".to_string(), "owner=agent".to_string()]).unwrap();
        assert_eq!(
            file_stem_prefix("sample files", &tags).unwrap(),
            "sample-files_suite-smoke-test_owner-agent"
        );
    }

    #[test]
    fn renders_generated_file_name_templates() {
        assert_eq!(
            generated_file_name(
                Some("{format}-{size}.{extension}"),
                "sample",
                "csv",
                "1KiB",
                "csv",
                2
            )
            .unwrap(),
            "csv-1KiB.csv"
        );
        assert_eq!(
            generated_file_name(None, "sample", "txt", "1KiB", "txt", 1).unwrap(),
            "sample_1KiB.txt"
        );
    }

    #[test]
    fn rejects_path_like_generated_file_names() {
        assert!(generated_file_name(Some("../x.txt"), "sample", "txt", "1KiB", "txt", 1).is_err());
        assert!(
            generated_file_name(Some("dir\\x.txt"), "sample", "txt", "1KiB", "txt", 1).is_err()
        );
    }

    #[test]
    fn serializes_tags_as_json() {
        let tags = parse_tags(&["suite=smoke".to_string()]).unwrap();
        assert_eq!(
            tags_json(&tags).unwrap(),
            r#"[{"key":"suite","value":"smoke"}]"#
        );
    }
}
