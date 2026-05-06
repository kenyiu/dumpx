use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;

#[test]
fn help_mentions_tags_and_agent_output() {
    Command::cargo_bin("dumpx")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("--tag"))
        .stdout(contains("--agent"))
        .stdout(contains("--quiet"))
        .stdout(contains("--output"))
        .stdout(contains("--json"))
        .stdout(contains("For agents"));
}

#[test]
fn lists_supported_formats() {
    Command::cargo_bin("dumpx")
        .unwrap()
        .arg("--list-formats")
        .assert()
        .success()
        .stdout(contains("csv"))
        .stdout(contains("parquet"))
        .stdout(contains("png"));
}

#[test]
fn generates_tagged_files_with_json_report() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "--out-dir",
            temp_dir.path().to_str().unwrap(),
            "--size",
            "1KiB",
            "--format",
            "csv,jsonl",
            "--tag",
            "suite=smoke",
            "--output",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["count"], 2);

    let files = report["files"].as_array().unwrap();
    for file in files {
        let path = file["path"].as_str().unwrap();
        assert!(path.contains("suite-smoke"));
        assert!(std::fs::metadata(path).unwrap().len() >= 1024);
        assert_eq!(file["tags"][0]["key"], "suite");
        assert_eq!(file["tags"][0]["value"], "smoke");
    }
}

#[test]
fn positional_format_then_size_is_supported() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "csv",
            "1KiB",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["count"], 1);
    assert_eq!(report["files"][0]["format"], "csv");
    assert_eq!(report["files"][0]["requested_size"], 1024);
}

#[test]
fn positional_size_then_format_is_supported() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "1KiB",
            "csv",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["count"], 1);
    assert_eq!(report["files"][0]["format"], "csv");
    assert_eq!(report["files"][0]["requested_size"], 1024);
}

#[test]
fn positional_comma_separated_formats_are_supported() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "csv,json",
            "1KiB",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["count"], 2);
    assert_eq!(report["files"][0]["format"], "csv");
    assert_eq!(report["files"][1]["format"], "json");
    assert_eq!(report["files"][0]["requested_size"], 1024);
}

#[test]
fn positional_comma_separated_formats_and_sizes_are_supported() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--json",
            "csv,json",
            "1KiB,2KiB",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["count"], 4);
    assert_eq!(report["files"][0]["format"], "csv");
    assert_eq!(report["files"][0]["requested_size"], 1024);
    assert_eq!(report["files"][1]["format"], "csv");
    assert_eq!(report["files"][1]["requested_size"], 2048);
    assert_eq!(report["files"][2]["format"], "json");
    assert_eq!(report["files"][2]["requested_size"], 1024);
    assert_eq!(report["files"][3]["format"], "json");
    assert_eq!(report["files"][3]["requested_size"], 2048);
}

#[test]
fn positional_output_dir_is_supported() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args(["--json", "csv", "1KiB", temp_dir.path().to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["count"], 1);
    assert_eq!(report["out_dir"], temp_dir.path().to_str().unwrap());
    let path = report["files"][0]["path"].as_str().unwrap();
    assert!(path.starts_with(temp_dir.path().to_str().unwrap()));
    assert!(std::fs::metadata(path).unwrap().len() >= 1024);
}

#[test]
fn default_output_dir_is_current_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .current_dir(temp_dir.path())
        .args(["--json", "txt", "1KiB"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["out_dir"], ".");
    let path = report["files"][0]["path"].as_str().unwrap();
    assert!(path.starts_with("./"));
    assert!(temp_dir.path().join(path.trim_start_matches("./")).exists());
}

#[test]
fn refuses_to_overwrite_existing_files_without_force() {
    let temp_dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["-o", temp_dir.path().to_str().unwrap(), "txt", "1KiB"])
        .assert()
        .success();

    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["-o", temp_dir.path().to_str().unwrap(), "txt", "1KiB"])
        .assert()
        .failure()
        .stderr(contains("failed to create"));
}

#[test]
fn force_allows_overwriting_existing_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["-o", temp_dir.path().to_str().unwrap(), "txt", "1KiB"])
        .assert()
        .success();

    Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--force",
            "txt",
            "1KiB",
        ])
        .assert()
        .success();
}

#[cfg(unix)]
#[test]
fn refuses_to_write_through_symlink_even_with_force() {
    use std::os::unix::fs::symlink;

    let temp_dir = tempfile::tempdir().unwrap();
    let target = temp_dir.path().join("target.txt");
    std::fs::write(&target, "keep").unwrap();
    symlink(&target, temp_dir.path().join("sample_1KiB.txt")).unwrap();

    Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--force",
            "txt",
            "1KiB",
        ])
        .assert()
        .failure()
        .stderr(contains("refusing to write through symlink"));
}

#[test]
fn rejects_ambiguous_positional_typo_instead_of_treating_it_as_output_dir() {
    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["--json", "cvs", "1KiB"])
        .assert()
        .failure()
        .stdout(contains(r#""type": "error""#))
        .stdout(contains("could not infer positional argument `cvs`"));
}

#[test]
fn rejects_sizes_above_default_limit_without_allow_large() {
    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["--json", "txt", "2GiB"])
        .assert()
        .failure()
        .stdout(contains(r#""ok": false"#))
        .stdout(contains("exceeds default limit"));
}

#[test]
fn rejects_runs_above_max_files() {
    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["--json", "--max-files", "1", "csv,json", "1KiB"])
        .assert()
        .failure()
        .stdout(contains(r#""type": "error""#))
        .stdout(contains("exceeds --max-files"));
}

#[test]
fn json_mode_errors_are_machine_readable() {
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args(["--json", "txt", "2GiB"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["type"], "error");
    assert_eq!(report["ok"], false);
    assert!(report["error"]
        .as_str()
        .unwrap()
        .contains("exceeds default limit"));
}

#[test]
fn csv_template_generates_fake_rows() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "--json",
            "csv",
            "1KiB",
            temp_dir.path().to_str().unwrap(),
            "--template-header",
            "id,name,email",
            "--template",
            "{{id}},{{name}},{{email}}",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    let path = report["files"][0]["path"].as_str().unwrap();
    let contents = std::fs::read_to_string(path).unwrap();
    assert!(contents.starts_with("id,name,email\n1,"));
    assert!(contents.contains("@"));
}

#[test]
fn jsonl_template_generates_json_objects() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "--json",
            "jsonl",
            "1KiB",
            temp_dir.path().to_str().unwrap(),
            "--template",
            r#"{"id":{{id}},"name":"{{name}}","email":"{{email}}"}"#,
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    let path = report["files"][0]["path"].as_str().unwrap();
    let contents = std::fs::read_to_string(path).unwrap();
    let first_line = contents.lines().next().unwrap();
    let row: Value = serde_json::from_str(first_line).unwrap();
    assert_eq!(row["id"], 1);
    assert!(row["email"].as_str().unwrap().contains("@"));
}

#[test]
fn template_file_is_supported() {
    let temp_dir = tempfile::tempdir().unwrap();
    let template_path = temp_dir.path().join("row.tmpl");
    std::fs::write(&template_path, "user {{id}} {{username}}").unwrap();

    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "--json",
            "txt",
            "1KiB",
            temp_dir.path().to_str().unwrap(),
            "--template-file",
            template_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    let path = report["files"][0]["path"].as_str().unwrap();
    let contents = std::fs::read_to_string(path).unwrap();
    assert!(contents.starts_with("user 1 "));
}

#[test]
fn invalid_template_fails_with_json_error() {
    Command::cargo_bin("dumpx")
        .unwrap()
        .args(["--json", "txt", "1KiB", "--template", "{{not_a_field}}"])
        .assert()
        .failure()
        .stdout(contains(r#""type": "error""#))
        .stdout(contains("unknown template placeholder"));
}

#[test]
fn quiet_json_output_is_agent_friendly() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "--out-dir",
            temp_dir.path().to_str().unwrap(),
            "--size",
            "1KiB",
            "--format",
            "txt",
            "--tag",
            "run=ci",
            "--quiet",
            "--output",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["type"], "summary");
    assert_eq!(report["count"], 1);
    assert_eq!(report["files"][0]["tags"][0]["key"], "run");
}

#[test]
fn json_flag_is_compact_agent_shorthand() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "-o",
            temp_dir.path().to_str().unwrap(),
            "-s",
            "1KiB",
            "-f",
            "txt",
            "-t",
            "run=ci",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["type"], "summary");
    assert_eq!(report["count"], 1);
    assert_eq!(report["files"][0]["tags"][0]["value"], "ci");
}

#[test]
fn deprecated_agent_flag_matches_quiet_json_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = Command::cargo_bin("dumpx")
        .unwrap()
        .args([
            "--out-dir",
            temp_dir.path().to_str().unwrap(),
            "--size",
            "1KiB",
            "--format",
            "txt",
            "--tag",
            "run=ci",
            "--agent",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["type"], "summary");
    assert_eq!(report["count"], 1);
}

#[test]
fn no_arguments_prompts_for_input() {
    let temp_dir = tempfile::tempdir().unwrap();
    let stdin = format!(
        "{}\n1KiB\ntxt\nsuite=prompt\nprompted\njson\n",
        temp_dir.path().display()
    );
    let assert = Command::cargo_bin("dumpx")
        .unwrap()
        .write_stdin(stdin)
        .assert()
        .success()
        .stderr(contains("No arguments provided"))
        .stderr(contains("Output directory"));

    let output = assert.get_output().stdout.clone();
    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["count"], 1);

    let path = report["files"][0]["path"].as_str().unwrap();
    assert!(path.contains("prompted_suite-prompt"));
    assert!(std::fs::metadata(path).unwrap().len() >= 1024);
}
