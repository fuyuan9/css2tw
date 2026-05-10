//! Custom cargo commands for project maintenance (xtask).
//!
//! This crate provides tasks for building distributions and publishing
//! to npm, avoiding the need for bash scripts.

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
        .args(&["build", "--release"])
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
