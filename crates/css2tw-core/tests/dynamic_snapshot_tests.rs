use css2tw_core::source::SourceFile;
use css2tw_core::Config;
use css2tw_core::Converter;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_dynamic_pattern_snapshots() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/dynamic");
    let snapshot_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/snapshots");

    if !snapshot_dir.exists() {
        fs::create_dir_all(&snapshot_dir).unwrap();
    }

    let entries = fs::read_dir(fixture_dir).expect("Failed to read fixture directory");

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let file_name = path.file_name().unwrap().to_str().unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let source = SourceFile {
            path: file_name.to_string(),
            content,
        };

        // Run conversion with a default config
        let config = Config::default();
        let converter = Converter::new(config);

        // We use empty style rules for testing dynamic detection logic itself
        let style_rules = vec![];
        let replacements = converter.plan_file(&source, &style_rules).unwrap();

        // Collect diagnostics and other info for snapshot
        let snapshot_data = serde_json::json!({
            "file": file_name,
            "unconverted_count": replacements.iter().filter(|r| r.after.is_empty()).count(),
            "diagnostics": replacements.iter()
                .flat_map(|r| r.diagnostics.clone())
                .collect::<Vec<_>>(),
            "replacements": replacements.iter().map(|r| {
                serde_json::json!({
                    "span": { "start": r.span.start, "end": r.span.end },
                    "before": r.before,
                    "after": r.after,
                    "confidence": r.confidence.score,
                    "failure_reason": r.failure_reason,
                    "suggestion": r.suggestion
                })
            }).collect::<Vec<_>>()
        });

        let snapshot_path = snapshot_dir.join(format!("{}.json", file_name));

        if snapshot_path.exists() {
            let existing_json = fs::read_to_string(&snapshot_path).unwrap();
            let existing_data: serde_json::Value = serde_json::from_str(&existing_json).unwrap();

            // Compare (normalized for line endings if necessary, but JSON should be fine)
            if snapshot_data != existing_data {
                // If they don't match, we fail the test and print a diff
                // In a real environment, we'd use something like `insta`
                let current_json = serde_json::to_string_pretty(&snapshot_data).unwrap();

                // Write to a temporary file for debugging
                let actual_path = snapshot_dir.join(format!("{}.actual.json", file_name));
                fs::write(&actual_path, &current_json).unwrap();

                panic!(
                    "Snapshot mismatch for {}. \nExpected: {}\nActual saved to: {:?}",
                    file_name, existing_json, actual_path
                );
            }
        } else {
            // Create initial snapshot
            let json = serde_json::to_string_pretty(&snapshot_data).unwrap();
            fs::write(&snapshot_path, json).unwrap();
            println!("Created initial snapshot for {}", file_name);
        }
    }
}
