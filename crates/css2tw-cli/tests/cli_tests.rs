use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn test_cli_compact_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("css2tw")?;
    cmd.arg("scan").arg(".").arg("--json").arg("--compact");
    
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    
    // Check if it's a single line (compact)
    assert_eq!(stdout.lines().count(), 1);
    // Should be valid JSON
    let _: serde_json::Value = serde_json::from_str(&stdout)?;
    
    Ok(())
}

#[test]
fn test_cli_no_trace() -> Result<(), Box<dyn std::error::Error>> {
    // We need some CSS and source to test conversion
    let temp = tempfile::tempdir()?;
    let css_path = temp.path().join("style.css");
    let html_path = temp.path().join("index.html");
    
    std::fs::write(&css_path, ".pt-16 { padding-top: 16px; }")?;
    std::fs::write(&html_path, r#"<div class="pt-16"></div>"#)?;
    
    let mut cmd = Command::cargo_bin("css2tw")?;
    cmd.arg("convert")
       .arg(temp.path())
       .arg("--json")
       .arg("--no-trace");
    
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    
    assert!(!stdout.contains("\"trace\""));
    
    Ok(())
}

#[test]
fn test_cli_summary_only() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let css_path = temp.path().join("style.css");
    let html_path = temp.path().join("index.html");
    
    std::fs::write(&css_path, ".pt-16 { padding-top: 16px; }")?;
    std::fs::write(&html_path, r#"<div class="pt-16"></div>"#)?;
    
    let mut cmd = Command::cargo_bin("css2tw")?;
    cmd.arg("convert")
       .arg(temp.path())
       .arg("--json")
       .arg("--summary-only");
    
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    
    assert!(!stdout.contains("\"changes\""));
    assert!(stdout.contains("\"summary\""));
    
    Ok(())
}

#[test]
fn test_cli_explain() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let css_path = temp.path().join("style.css");
    std::fs::write(&css_path, ".pt-16 { padding-top: 16px; }")?;
    
    let mut cmd = Command::cargo_bin("css2tw")?;
    cmd.arg("explain")
       .arg(".pt-16")
       .arg("--css")
       .arg(&css_path)
       .arg("--json");
    
    let output = cmd.output()?;
    let stdout = String::from_utf8(output.stdout)?;
    
    let json: serde_json::Value = serde_json::from_str(&stdout)?;
    assert_eq!(json["selector"], ".pt-16");
    assert_eq!(json["convertible"], true);
    assert_eq!(json["after"], "pt-4");
    assert!(json["trace"].is_array());
    
    Ok(())
}
