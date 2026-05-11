use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_convert_with_external_css() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("index.html");
    fs::write(&source_path, r#"<div class="pt-16"></div>"#).unwrap();

    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    cmd.arg("convert")
        .arg(dir.path())
        .arg("--css-inline")
        .arg(".pt-16 { padding-top: 16px; }")
        .arg("--write");

    cmd.assert().success();

    let content = fs::read_to_string(source_path).unwrap();
    assert!(content.contains("pt-4"));
}

#[test]
fn test_cli_scan_with_external_css() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("index.php");
    fs::write(&source_path, r#"<div class="pt-16"></div>"#).unwrap();

    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    cmd.arg("scan")
        .arg(dir.path())
        .arg("--css-inline")
        .arg(".pt-16 { padding-top: 16px; }")
        .arg("--json");

    let output = cmd.assert().success().get_output().stdout.clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    assert_eq!(json["summary"]["replacements_planned"], 1);
}
