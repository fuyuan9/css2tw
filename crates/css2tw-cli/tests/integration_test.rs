use assert_cmd::Command;

#[test]
fn test_cli_scan() {
    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let assert = cmd
        .arg("scan")
        .arg("tests/fixtures")
        .arg("--css-file")
        .arg("tests/fixtures/sample.css")
        .arg("--json")
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    // files_scanned no longer counts .css files
    assert!(output.contains("\"files_scanned\":4"));
}

#[test]
fn test_cli_convert_complex() {
    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let assert = cmd
        .arg("convert")
        .arg("tests/fixtures")
        .arg("--css-file")
        .arg("tests/fixtures/comprehensive.css")
        .arg("--json")
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Check if HTML file was scanned
    assert!(output.contains("sample.html"));
    // Check for complex conversion result (before:mr-2)
    assert!(output.contains("before:mr-2"));
    // Check for hover variant
    assert!(output.contains("hover:bg-[#00008b]"));
}

#[test]
fn test_cli_advanced_html_conversion() {
    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let assert = cmd
        .arg("convert")
        .arg("tests/fixtures/advanced")
        .arg("--css-file")
        .arg("tests/fixtures/advanced/styles.css")
        .arg("--json")
        .arg("--rem-scale")
        .arg("4.0")
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Tag specific: input.text-box should have border, div.text-box should have bg
    assert!(output.contains("border-[1px_solid_gray]")); // from input.text-box
    assert!(output.contains("bg-[#fff]")); // from div.text-box (white -> #fff)

    // Combinators: .container .child should have ml-4 (1rem * 4)
    assert!(output.contains("ml-4"));

    // Combinators: .list > li should have pb-2 (0.5rem * 4)
    assert!(output.contains("pb-2"));

    // Specificity: #unique-header should override .header
    assert!(output.contains("text-8"));
    assert!(output.contains("font-bold"));
}

#[test]
fn test_cli_advanced_jsx_conversion() {
    let mut cmd = Command::cargo_bin("css2tw").unwrap();
    let assert = cmd
        .arg("convert")
        .arg("tests/fixtures/advanced")
        .arg("--css-file")
        .arg("tests/fixtures/advanced/styles.css")
        .arg("--json")
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // Check JSX file
    assert!(output.contains("App.jsx"));
    // Tag specific in JSX
    assert!(output.contains("bg-[#fff]")); // for <div className="text-box">
}

#[test]
fn test_cli_rem_scale_variation() {
    // Test with scale 4.0
    let mut cmd4 = Command::cargo_bin("css2tw").unwrap();
    let out4 = String::from_utf8(
        cmd4.arg("convert")
            .arg("tests/fixtures/advanced")
            .arg("--css-file")
            .arg("tests/fixtures/advanced/styles.css")
            .arg("--rem-scale")
            .arg("4.0")
            .arg("--json")
            .unwrap()
            .stdout,
    )
    .unwrap();

    // 2.5rem * 4.0 = 10 -> w-10
    assert!(out4.contains("w-10"));

    // Test with scale 5.0
    let mut cmd5 = Command::cargo_bin("css2tw").unwrap();
    let out5 = String::from_utf8(
        cmd5.arg("convert")
            .arg("tests/fixtures/advanced")
            .arg("--css-file")
            .arg("tests/fixtures/advanced/styles.css")
            .arg("--rem-scale")
            .arg("5.0")
            .arg("--json")
            .unwrap()
            .stdout,
    )
    .unwrap();

    // 2.5rem * 5.0 = 12.5 -> w-12.5
    assert!(out5.contains("w-12.5"));
}
