use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_stdin_support() {
    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let input = r#"<div class="custom-btn"></div>"#;

    let output = cmd
        .arg("convert")
        .arg("--stdin")
        .arg("--stdin-type")
        .arg("html")
        .arg("--css-inline")
        .arg(".custom-btn { padding: 10px; }")
        .arg("--json")
        .arg("--include-patched")
        .write_stdin(input)
        .unwrap()
        .stdout;

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    // Debug print
    if !json["changes"][0]["patched_content"]
        .as_str()
        .is_some_and(|p| p.contains("p-2.5"))
    {
        println!(
            "JSON Output: {}",
            serde_json::to_string_pretty(&json).unwrap()
        );
    }

    // Check if patched_content contains the tailwind class
    let patched = json["changes"][0]["patched_content"]
        .as_str()
        .expect("Missing patched_content");
    assert!(
        patched.contains("p-2.5") || patched.contains("p-[10px]"),
        "Expected p-2.5 or p-[10px] in patched content, found: {}",
        patched
    );
    assert_eq!(json["changes"][0]["file"], "stdin.html");
}

#[test]
fn test_cli_ndjson_streaming() {
    let dir = tempdir().unwrap();
    let file1 = dir.path().join("a.html");
    fs::write(&file1, r#"<div class="a"></div>"#).unwrap();

    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let output = cmd
        .arg("scan")
        .arg(dir.path())
        .arg("--css-inline")
        .arg(".a { color: red; }")
        .arg("--ndjson")
        .unwrap()
        .stdout;

    let stdout_str = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = stdout_str.trim().split('\n').collect();

    assert!(lines.len() >= 3); // start, file, summary

    let start: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(start["type"], "start");

    let file: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(file["type"], "file");
    assert_eq!(file["data"]["file"], file1.to_str().unwrap());

    let summary: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(summary["type"], "summary");
}

#[test]
fn test_cli_raw_css_and_suggestions() {
    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let input = r#"<div class="complex"></div>"#;

    let output = cmd
        .arg("convert")
        .arg("--stdin")
        .arg("--css-inline")
        .arg(".complex { display: flex; align-items: baseline; }")
        .arg("--json")
        .write_stdin(input)
        .unwrap()
        .stdout;

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();

    // Should be in unconverted because flex + align-items: baseline might not map to a single utility in some versions
    // Actually flex maps to 'flex'. align-items: baseline maps to 'items-baseline'.
    // So it should be in 'changes' if it maps.

    let rep = if !json["changes"].as_array().unwrap().is_empty() {
        &json["changes"][0]["patches"][0]
    } else {
        &json["unconverted"][0]
    };

    // Should capture raw CSS
    assert!(
        rep["raw_css"].as_str().is_some(),
        "Expected raw_css to be populated"
    );
    assert!(rep["raw_css"].as_str().unwrap().contains("display"));
}
