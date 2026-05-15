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
        Some("bench-bootstrap") => bench_bootstrap()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
dist            Builds the project and copies the binary to npm/platforms
publish         Publishes all packages to npm
publish-dry-run Simulates npm publish for all packages
bench-bootstrap Runs a conversion benchmark against Bootstrap CSS
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

/// Runs a conversion benchmark against Bootstrap CSS.
fn bench_bootstrap() -> Result<(), DynError> {
    let root = project_root();
    let bootstrap_path = root.join("fixtures").join("benchmarks").join("bootstrap.css");

    if !bootstrap_path.exists() {
        return Err(format!(
            "Bootstrap CSS not found at {}. Please run `curl -sSL https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.css -o {}`",
            bootstrap_path.display(),
            bootstrap_path.display()
        ).into());
    }

    println!("{}", "=== Bootstrap Conversion Benchmark ===".bold().cyan());
    println!("Loading {}...", bootstrap_path.display());

    let css_content = fs::read_to_string(&bootstrap_path)?;
    let parsed = parse_css(&css_content).map_err(|e| format!("Failed to parse CSS: {:?}", e))?;
    let rules = extract_style_rules(&parsed);
    let rule_map = build_rule_map(&rules);
    let variable_map = extract_variables(&rules);

    let total_classes = rule_map.len();
    let mut safe_conversions = 0;
    let mut partial_conversions = 0;
    let mut failed_conversions = 0;

    println!("Analyzing {} unique classes...\n", total_classes);

    for class_name in rule_map.keys() {
        let replacement = ConversionPlanner::explain(class_name, &rule_map, &variable_map, 4.0);

        match replacement {
            Some(rep) => {
                if !rep.after.is_empty() && rep.failure_reason.is_none() && rep.confidence.score >= 1.0 {
                    safe_conversions += 1;
                } else if !rep.after.is_empty() {
                    partial_conversions += 1;
                } else {
                    failed_conversions += 1;
                }
            }
            None => {
                failed_conversions += 1;
            }
        }
    }

    let safe_pct = (safe_conversions as f64 / total_classes as f64) * 100.0;
    let partial_pct = (partial_conversions as f64 / total_classes as f64) * 100.0;
    let failed_pct = (failed_conversions as f64 / total_classes as f64) * 100.0;

    println!("{:<20} : {:>5} ({:>6.2}%)", "Safe Conversions".green(), safe_conversions, safe_pct);
    println!("{:<20} : {:>5} ({:>6.2}%)", "Partial".yellow(), partial_conversions, partial_pct);
    println!("{:<20} : {:>5} ({:>6.2}%)", "Failed".red(), failed_conversions, failed_pct);
    println!("{:-<40}", "");
    println!("{:<20} : {:>5}", "Total Classes".bold(), total_classes);

    Ok(())
}
