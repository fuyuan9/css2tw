//! Custom cargo commands for project maintenance (xtask).
//!
//! This crate provides tasks for building distributions and publishing
//! to npm, avoiding the need for bash scripts.

use colored::Colorize;
use css2tw_core::css::parser::{build_rule_map, extract_style_rules, extract_variables, parse_css};
use css2tw_core::rewrite::planner::ConversionPlanner;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("dist") => dist()?,
        Some("publish-dry-run") => publish(true)?,
        Some("publish") => publish(false)?,
        Some("bench-bootstrap") | Some("bench-bootstrap-v5") => bench_bootstrap_v5()?,
        Some("bench-bootstrap-v4") => bench_bootstrap_v4()?,
        Some("bench-bootstrap-v3") => bench_bootstrap_v3()?,
        Some("bench-bulma") | Some("bench-bulma-v0") => bench_bulma_v0()?,
        Some("bench-bulma-v1") => bench_bulma_v1()?,
        Some("bench-foundation") => bench_foundation()?,
        Some("bench-skeleton") => bench_skeleton()?,
        Some("bench-uikit") => bench_uikit()?,
        Some("bench-semantic") => bench_semantic()?,
        Some("bench-all") => bench_all()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
dist                Builds the project and copies the binary to npm/platforms
publish             Publishes all packages to npm
publish-dry-run     Simulates npm publish for all packages
bench-bootstrap-v5  Runs benchmark against Bootstrap v5
bench-bootstrap-v4  Runs benchmark against Bootstrap v4
bench-bootstrap-v3  Runs benchmark against Bootstrap v3
bench-bulma-v0      Runs benchmark against Bulma v0.9
bench-bulma-v1      Runs benchmark against Bulma v1.0
bench-foundation    Runs benchmark against Foundation v6
bench-skeleton      Runs benchmark against Skeleton v2
bench-uikit         Runs benchmark against UIkit v3
bench-semantic      Runs benchmark against Semantic UI (Fomantic)
bench-all           Runs all benchmarks and shows a summary
"
    )
}

/// Builds the release binary and copies it to the appropriate npm platform directory.
fn dist() -> Result<(), DynError> {
    let root = project_root();

    // 1. Build
    println!("Building release binary...");
    let status = Command::new("cargo")
        .current_dir(&root)
        .args(["build", "--release"])
        .status()?;

    if !status.success() {
        return Err("Cargo build failed".into());
    }

    // 2. Identify platform
    let os = env::consts::OS; // linux, macos, windows
    let arch = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };

    let platform_key = match os {
        "macos" => format!("darwin-{}", arch),
        "windows" => format!("win32-{}", arch),
        other => format!("{}-{}", other, arch),
    };

    let exe_name = if os == "windows" {
        "css2tw.exe"
    } else {
        "css2tw"
    };
    let source = root.join("target").join("release").join(exe_name);
    let dest_dir = root
        .join("npm")
        .join("platforms")
        .join(&platform_key)
        .join("bin");
    let dest = dest_dir.join(exe_name);

    println!("Copying {} to {}...", source.display(), dest.display());
    fs::create_dir_all(&dest_dir)?;
    fs::copy(&source, &dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;
    }

    println!(
        "Distribution package for {} prepared successfully!",
        platform_key
    );
    Ok(())
}

/// Publishes the npm packages (platform-specific and main wrapper).
fn publish(dry_run: bool) -> Result<(), DynError> {
    let root = project_root();
    let npm_dir = root.join("npm");
    let platforms_dir = npm_dir.join("platforms");

    // Publish platform packages
    if platforms_dir.exists() {
        for entry in fs::read_dir(platforms_dir)? {
            let path = entry?.path();
            if path.is_dir() {
                run_npm_publish(&path, dry_run)?;
            }
        }
    }

    // Publish main package
    run_npm_publish(&npm_dir, dry_run)?;

    Ok(())
}

fn run_npm_publish(dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let mut args = vec!["publish"];
    if dry_run {
        args.push("--dry-run");
    }

    println!(
        "\n--- {} publish in {} ---",
        if dry_run { "Dry-run" } else { "Real" },
        dir.display()
    );
    let status = Command::new("npm").current_dir(dir).args(&args).status()?;

    if !status.success() {
        return Err(format!("npm publish failed in {}", dir.display()).into());
    }
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

/// Runs all benchmarks.
fn bench_all() -> Result<(), DynError> {
    println!("{}", "=== Running All Benchmarks ===".bold().magenta());
    bench_bootstrap_v5()?;
    bench_bootstrap_v4()?;
    bench_bootstrap_v3()?;
    bench_bulma_v0()?;
    bench_bulma_v1()?;
    bench_foundation()?;
    bench_skeleton()?;
    bench_uikit()?;
    bench_semantic()?;
    println!(
        "{}",
        "\nAll benchmarks completed successfully!".bold().green()
    );
    Ok(())
}

fn bench_bootstrap_v5() -> Result<(), DynError> {
    let root = project_root();
    let path = root
        .join("fixtures")
        .join("benchmarks")
        .join("bootstrap5.css");
    run_benchmark("Bootstrap v5", &path)
}

fn bench_bootstrap_v4() -> Result<(), DynError> {
    let root = project_root();
    let path = root
        .join("fixtures")
        .join("benchmarks")
        .join("bootstrap4.css");
    run_benchmark("Bootstrap v4", &path)
}

fn bench_bootstrap_v3() -> Result<(), DynError> {
    let root = project_root();
    let path = root
        .join("fixtures")
        .join("benchmarks")
        .join("bootstrap3.css");
    run_benchmark("Bootstrap v3", &path)
}

fn bench_bulma_v0() -> Result<(), DynError> {
    let root = project_root();
    let path = root.join("fixtures").join("benchmarks").join("bulma0.css");
    run_benchmark("Bulma v0.9", &path)
}

fn bench_bulma_v1() -> Result<(), DynError> {
    let root = project_root();
    let path = root.join("fixtures").join("benchmarks").join("bulma1.css");
    run_benchmark("Bulma v1.0", &path)
}

fn bench_foundation() -> Result<(), DynError> {
    let root = project_root();
    let path = root
        .join("fixtures")
        .join("benchmarks")
        .join("foundation.css");
    run_benchmark("Foundation v6", &path)
}

fn bench_skeleton() -> Result<(), DynError> {
    let root = project_root();
    let path = root
        .join("fixtures")
        .join("benchmarks")
        .join("skeleton.css");
    run_benchmark("Skeleton v2", &path)
}

fn bench_uikit() -> Result<(), DynError> {
    let root = project_root();
    let path = root.join("fixtures").join("benchmarks").join("uikit.css");
    run_benchmark("UIkit v3", &path)
}

fn bench_semantic() -> Result<(), DynError> {
    let root = project_root();
    let path = root
        .join("fixtures")
        .join("benchmarks")
        .join("semantic.css");
    run_benchmark("Semantic UI", &path)
}

/// Generic benchmark runner.
fn run_benchmark(name: &str, css_path: &Path) -> Result<(), DynError> {
    if !css_path.exists() {
        return Err(format!("Benchmark file not found: {}", css_path.display()).into());
    }

    println!(
        "\n{}",
        format!("--- {} Conversion Benchmark ---", name)
            .bold()
            .cyan()
    );
    println!("Loading {}...", css_path.display());

    let css_content = fs::read_to_string(css_path)?;
    let parsed = parse_css(&css_content).map_err(|e| format!("Failed to parse CSS: {:?}", e))?;
    let rules = extract_style_rules(&parsed);
    let rule_map = build_rule_map(&rules);
    let variable_map = extract_variables(&rules);

    let total_classes = rule_map.len();
    let mut safe_conversions = 0;
    let mut partial_conversions = 0;
    let mut failed_conversions = 0;

    println!("Analyzing {} unique classes...", total_classes);

    let mut failure_reasons = std::collections::HashMap::new();
    let mut failed_examples = Vec::new();

    for class_name in rule_map.keys() {
        let replacement = ConversionPlanner::explain(class_name, &rule_map, &variable_map, 4.0);
        match replacement {
            Some(rep) => {
                if !rep.after.is_empty()
                    && rep.failure_reason.is_none()
                    && rep.confidence.score >= 1.0
                {
                    safe_conversions += 1;
                } else if rep.confidence.score > 0.0 {
                    partial_conversions += 1;

                    if rep.confidence.score < 1.0 {
                        for reason in &rep.confidence.reasons {
                            match reason {
                                css2tw_core::report::ConfidenceReason::VariableResolved(v) => {
                                    *failure_reasons
                                        .entry(format!("Partial (Var): {}", v))
                                        .or_insert(0) += 1;
                                }
                                css2tw_core::report::ConfidenceReason::PartialMatch(_) => {
                                    *failure_reasons
                                        .entry("Partial (PartialMatch)".to_string())
                                        .or_insert(0) += 1;
                                }
                                css2tw_core::report::ConfidenceReason::LowConfidenceProperty(p) => {
                                    *failure_reasons
                                        .entry(format!("Partial (LowConf): {}", p))
                                        .or_insert(0) += 1;
                                }
                                _ => {
                                    *failure_reasons
                                        .entry("Partial (Other)".to_string())
                                        .or_insert(0) += 1;
                                }
                            }
                        }
                    }

                    // Track unmapped properties for reporting
                    for t in &rep.trace {
                        if t.starts_with("Unmapped properties: ") {
                            let props = t.strip_prefix("Unmapped properties: ").unwrap();
                            for prop in props.split(", ") {
                                *failure_reasons
                                    .entry(format!("Partial Prop: {}", prop))
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                } else {
                    failed_conversions += 1;
                    let reason = rep
                        .failure_reason
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    *failure_reasons.entry(reason).or_insert(0) += 1;

                    // Track unmapped properties for reporting
                    for t in &rep.trace {
                        if t.starts_with("Unmapped properties: ") {
                            let props = t.strip_prefix("Unmapped properties: ").unwrap();
                            for prop in props.split(", ") {
                                *failure_reasons
                                    .entry(format!("Failed Prop: {}", prop))
                                    .or_insert(0) += 1;
                            }
                        }
                    }

                    if failed_examples.len() < 5 {
                        failed_examples.push(format!("{:<20} ({})", class_name, rep.after));
                    }
                }
            }
            None => {
                failed_conversions += 1;
                *failure_reasons
                    .entry("NoMappingFound".to_string())
                    .or_insert(0) += 1;
            }
        }
    }

    let safe_pct = (safe_conversions as f64 / total_classes as f64) * 100.0;
    let partial_pct = (partial_conversions as f64 / total_classes as f64) * 100.0;
    let failed_pct = (failed_conversions as f64 / total_classes as f64) * 100.0;

    println!(
        "{:<20} : {:>5} ({:>6.2}%)",
        "Safe Conversions".green(),
        safe_conversions,
        safe_pct
    );
    println!(
        "{:<20} : {:>5} ({:>6.2}%)",
        "Partial".yellow(),
        partial_conversions,
        partial_pct
    );
    println!(
        "{:<20} : {:>5} ({:>6.2}%)",
        "Failed".red(),
        failed_conversions,
        failed_pct
    );

    if !failure_reasons.is_empty() && (failed_conversions > 0 || partial_conversions > 0) {
        print!("  Reasons: ");
        let mut sorted_reasons: Vec<_> = failure_reasons.into_iter().collect();
        sorted_reasons.sort_by_key(|b| std::cmp::Reverse(b.1));
        let reason_strings: Vec<_> = sorted_reasons
            .into_iter()
            .take(20)
            .map(|(r, c)| format!("{} ({})", r, c))
            .collect();
        println!("{}", reason_strings.join(", "));
    }

    Ok(())
}
