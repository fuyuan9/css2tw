//! Command-line interface for css2tw.
//!
//! Provides commands for scanning projects, explaining conversions,
//! and migrating CSS to Tailwind utility classes.

use clap::{Parser, Subcommand};
use colored::*;
use css2tw_core::Config;

#[derive(Parser)]
#[command(name = "css2tw")]
#[command(about = "Production-grade CSS-to-Tailwind Migration CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output in JSON format
    #[arg(long, global = true)]
    json: bool,

    /// Enable color output
    #[arg(long, global = true)]
    color: bool,

    /// Include conversion trace in output
    #[arg(long, global = true)]
    trace: bool,

    /// Include conversion reasons in output
    #[arg(long, global = true)]
    reasons: bool,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pretty: bool,

    /// Include patched content in JSON output
    #[arg(long, global = true)]
    include_patched: bool,

    /// Use Newline Delimited JSON (NDJSON) output format
    #[arg(long, global = true)]
    ndjson: bool,

    /// Filter detailed report to specific files (summary remains global)
    #[arg(long, global = true)]
    file_only: Vec<String>,

    /// Read source from stdin
    #[arg(long, global = true)]
    stdin: bool,

    /// File type for stdin (e.g., html, jsx)
    #[arg(long, global = true)]
    stdin_type: Option<String>,

    /// Display diff of changes
    #[arg(long, global = true)]
    diff: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a repository and report convertible classes without writing files
    Scan {
        /// Path to scan
        #[arg(default_value = ".")]
        path: String,
        /// Return only the summary
        #[arg(long)]
        summary_only: bool,
        /// Additional CSS file to use for conversion
        #[arg(long)]
        css_file: Vec<String>,
        /// Inline CSS string to use for conversion
        #[arg(long)]
        css_inline: Option<String>,
        /// Include tag selectors (e.g. div) in conversion
        #[arg(long, default_value_t = false)]
        include_tag_selectors: bool,
    },
    /// Perform migration planning and optionally write changes
    Convert {
        /// Path to scan
        #[arg(default_value = ".")]
        path: String,
        /// Dry run mode (don't write files)
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        /// Write mode (overwrite files)
        #[arg(long, default_value_t = false)]
        write: bool,
        /// Confidence threshold
        #[arg(long, default_value_t = 0.8)]
        confidence_threshold: f64,
        /// Rem to Tailwind scale factor
        #[arg(long, default_value_t = 4.0)]
        rem_scale: f32,
        /// Custom theme values in key=value format (can be used multiple times)
        #[arg(long)]
        custom_theme: Vec<String>,
        /// Custom configuration in JSON format
        #[arg(long)]
        config_json: Option<String>,
        /// Return only the summary
        #[arg(long)]
        summary_only: bool,
        /// Additional CSS file to use for conversion (can be used multiple times)
        #[arg(long)]
        css_file: Vec<String>,
        /// Inline CSS string to use for conversion
        #[arg(long)]
        css_inline: Option<String>,
        /// Include tag selectors (e.g. div) in conversion
        #[arg(long, default_value_t = false)]
        include_tag_selectors: bool,
    },
    /// Explain how a CSS class would be converted
    Explain {
        /// CSS class selector (e.g. .btn-primary)
        selector: String,

        /// Path to the CSS file containing the class
        #[arg(long)]
        css: String,
    },
    /// Print resolved configuration
    Config,
    /// Print JSON schema for reports and config
    Schema,
    /// Detect and analyze Tailwind configuration in the current project
    DetectConfig {
        /// Path to the project root
        #[arg(default_value = ".")]
        path: String,
    },
    /// Run a benchmark on the project to measure conversion performance and accuracy
    Benchmark {
        /// Path to the directory or file to benchmark
        #[arg(default_value = ".")]
        path: String,
        /// Confidence threshold for conversion (0.0 to 1.0)
        #[arg(long, default_value = "0.7")]
        threshold: f64,
        /// Recursive search for files
        #[arg(long, default_value = "true")]
        recursive: bool,
    },
    /// Visual Regression Testing utilities
    Vrt {
        #[command(subcommand)]
        action: VrtAction,
    },
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum VrtAction {
    /// Initialize a Playwright-based VRT setup in the current directory
    Init {
        /// Target URL for capture (e.g., http://localhost:3000)
        #[arg(long, default_value = "http://localhost:3000")]
        url: String,
    },
}

/// Main entry point for the CLI application.
/// Parses arguments and dispatches to the appropriate command handler.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if !cli.color {
        control::set_override(false);
    }

    match &cli.command {
        Commands::Scan {
            path,
            summary_only,
            css_file,
            css_inline,
            include_tag_selectors,
        } => {
            process_migration(
                MigrationOptions {
                    path,
                    mode: "read_only",
                    confidence_threshold: 0.8,
                    rem_scale: 4.0,
                    custom_theme: &[],
                    config_json: None,
                    summary_only: *summary_only,
                    extra_css_files: css_file,
                    inline_css: css_inline.as_ref(),
                    include_tag_selectors: *include_tag_selectors,
                },
                &cli,
            )?;
        }
        Commands::Convert {
            path,
            dry_run,
            write,
            confidence_threshold,
            rem_scale,
            custom_theme,
            config_json,
            summary_only,
            css_file,
            css_inline,
            include_tag_selectors,
        } => {
            let mode = if *write && !*dry_run {
                "write"
            } else {
                "dry_run"
            };
            process_migration(
                MigrationOptions {
                    path,
                    mode,
                    confidence_threshold: *confidence_threshold,
                    rem_scale: *rem_scale,
                    custom_theme,
                    config_json: config_json.as_ref(),
                    summary_only: *summary_only,
                    extra_css_files: css_file,
                    inline_css: css_inline.as_ref(),
                    include_tag_selectors: *include_tag_selectors,
                },
                &cli,
            )?;
        }
        Commands::Explain { selector, css } => {
            let content = std::fs::read_to_string(css)?;
            let stylesheet = css2tw_core::css::parser::parse_css(&content)
                .map_err(|e| anyhow::anyhow!("CSS Parse Error: {:?}", e))?;
            let rules = css2tw_core::css::parser::extract_style_rules(&stylesheet);
            let rule_map = css2tw_core::css::parser::build_rule_map(&rules);
            let variable_map = css2tw_core::css::parser::extract_variables(&rules);

            let selector_clean = selector.trim_start_matches('.');
            let explanation = css2tw_core::rewrite::planner::ConversionPlanner::explain(
                selector_clean,
                &rule_map,
                &variable_map,
                4.0, // default rem_scale
            );

            if cli.json {
                let result = if let Some(exp) = explanation {
                    serde_json::json!({
                        "selector": selector,
                        "convertible": true,
                        "before": exp.before,
                        "after": exp.after,
                        "confidence": exp.confidence,
                        "trace": exp.trace,
                    })
                } else {
                    serde_json::json!({
                        "selector": selector,
                        "convertible": false,
                        "reason": "Selector not found or no tailwind mappings available"
                    })
                };
                if cli.pretty {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("{}", serde_json::to_string(&result)?);
                }
            } else {
                match explanation {
                    Some(exp) => {
                        println!("Selector: {}", selector);
                        println!("Result:   {}", exp.after);
                        println!("Trace:");
                        for t in exp.trace {
                            println!("  - {}", t);
                        }
                    }
                    None => {
                        println!("Could not explain selector: {}", selector);
                    }
                }
            }
        }
        Commands::Config => {
            let config = Config::default();
            if cli.json {
                if cli.pretty {
                    println!("{}", serde_json::to_string_pretty(&config)?);
                } else {
                    println!("{}", serde_json::to_string(&config)?);
                }
            } else {
                println!("{:#?}", config);
            }
        }
        Commands::Schema => {
            let report_schema = schemars::schema_for!(css2tw_core::report::Report);
            let config_schema = schemars::schema_for!(css2tw_core::Config);

            let combined = serde_json::json!({
                "report": report_schema,
                "config": config_schema
            });

            if cli.pretty {
                println!("{}", serde_json::to_string_pretty(&combined)?);
            } else {
                println!("{}", serde_json::to_string(&combined)?);
            }
        }
        Commands::DetectConfig { path } => {
            let result =
                css2tw_core::tailwind::detector::ConfigDetector::detect(std::path::Path::new(path));
            if cli.json {
                if cli.pretty {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("{}", serde_json::to_string(&result)?);
                }
            } else {
                if let Some(file) = &result.file_found {
                    println!(
                        "{} Found Tailwind config: {}",
                        "Success".green().bold(),
                        file.bold()
                    );
                    println!("Spacing tokens: {}", result.spacing.len());
                    println!("Color tokens:   {}", result.colors.len());
                    println!("Screens:        {}", result.screens.len());
                } else {
                    println!(
                        "{} No Tailwind config found in {}",
                        "Warning".yellow().bold(),
                        path
                    );
                }
            }
        }
        Commands::Benchmark {
            path,
            threshold,
            recursive: _recursive,
        } => {
            let start = std::time::Instant::now();
            let paths = if std::path::Path::new(path).is_file() {
                vec![std::path::PathBuf::from(path)]
            } else {
                css2tw_core::source::Scanner::scan_directory(path).unwrap_or_default()
            };

            let mut total_classes = 0;
            let mut converted_classes = 0;
            let mut failure_distribution = std::collections::HashMap::new();
            let mut diagnostics_count = 0;

            let source_files = css2tw_core::source::Scanner::read_files_parallel(&paths);
            let mut css_contents = Vec::new();
            for sf in &source_files {
                if sf.path.ends_with(".css") {
                    css_contents.push(sf.content.clone());
                }
            }

            let config = css2tw_core::Config {
                confidence_threshold: *threshold as f32,
                ..Default::default()
            };
            let converter = css2tw_core::Converter::new(config);

            for sf in &source_files {
                if sf.path.ends_with(".css") {
                    continue;
                }
                if let Ok(reps) = converter.plan_file(sf, &css_contents) {
                    total_classes += reps.len();
                    for rep in reps {
                        if rep.failure_reason.is_none() && rep.confidence.score as f64 >= *threshold
                        {
                            converted_classes += 1;
                        } else if let Some(reason) = rep.failure_reason {
                            let reason_str = format!("{:?}", reason);
                            *failure_distribution.entry(reason_str).or_insert(0) += 1;
                        }
                        diagnostics_count += rep.diagnostics.len();
                    }
                }
            }

            let elapsed = start.elapsed();
            let result = css2tw_core::report::BenchmarkResult {
                total_files: source_files.len(),
                total_time_ms: elapsed.as_millis(),
                avg_time_per_file_ms: if !source_files.is_empty() {
                    elapsed.as_millis() as f64 / source_files.len() as f64
                } else {
                    0.0
                },
                total_classes_found: total_classes,
                total_classes_converted: converted_classes,
                conversion_rate: if total_classes > 0 {
                    converted_classes as f64 / total_classes as f64
                } else {
                    0.0
                },
                failure_distribution,
                diagnostics_count,
            };

            if cli.json {
                if cli.pretty {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!("{}", serde_json::to_string(&result)?);
                }
            } else {
                println!("\n{}", "Benchmark Results".bold().underline());
                println!("Total Files:        {}", result.total_files);
                println!("Total Time:         {}ms", result.total_time_ms);
                println!("Avg Time/File:      {:.2}ms", result.avg_time_per_file_ms);
                println!("Classes Found:      {}", result.total_classes_found);
                println!("Classes Converted:  {}", result.total_classes_converted);
                println!("Conversion Rate:    {:.2}%", result.conversion_rate * 100.0);
                println!("Diagnostics issued: {}", result.diagnostics_count);

                if !result.failure_distribution.is_empty() {
                    println!("\n{}", "Failure Distribution:".bold());
                    let mut sorted_failures: Vec<_> = result.failure_distribution.iter().collect();
                    sorted_failures.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
                    for (reason, count) in sorted_failures {
                        println!("  - {:<20}: {}", reason, count);
                    }
                }
            }
        }
        Commands::Vrt { action } => match action {
            VrtAction::Init { url } => {
                println!(
                    "{} Initializing Visual Regression setup...",
                    "VRT".bold().blue()
                );

                let _ = std::fs::create_dir_all("tests-vrt");

                let config_content = format!(
                    r#"import {{ defineConfig, devices }} from '@playwright/test';

export default defineConfig({{
  testDir: './tests-vrt',
  fullyParallel: true,
  reporter: 'html',
  use: {{
    baseURL: '{}',
    trace: 'on-first-retry',
  }},
  projects: [
    {{
      name: 'chromium',
      use: {{ ...devices['Desktop Chrome'] }},
    }},
  ],
}});
"#,
                    url
                );

                let test_content = format!(
                    r#"import {{ test, expect }} from '@playwright/test';

test('Visual Regression Comparison', async ({{ page }}) => {{
  await page.goto('/');
  await page.waitForLoadState('networkidle');

  // Baseline snapshot
  // Run with `npx playwright test -c playwright.vrt.config.ts --update-snapshots` first
  // Then run migration, and run `npx playwright test -c playwright.vrt.config.ts` again to compare
  await expect(page).toHaveScreenshot('site-baseline.png', {{
    fullPage: true,
    maxDiffPixelRatio: 0.01,
  }});
}});
"#
                );

                std::fs::write("playwright.vrt.config.ts", config_content)?;
                std::fs::write("tests-vrt/migrate-vrt.spec.ts", test_content)?;

                println!(
                    "{} Created playwright.vrt.config.ts and tests-vrt/migrate-vrt.spec.ts",
                    "Success".green().bold()
                );
                println!("\nNext steps for VRT:");
                println!(
                    "1. Install Playwright: {}",
                    "npm install -D @playwright/test".cyan()
                );
                println!("2. Start your dev server (e.g. at {})", url.bold());
                println!(
                    "3. Capture baseline:   {}",
                    "npx playwright test -c playwright.vrt.config.ts --update-snapshots".cyan()
                );
                println!(
                    "4. Run migration:      {}",
                    "css2tw convert . --write".cyan()
                );
                println!(
                    "5. Compare results:    {}",
                    "npx playwright test -c playwright.vrt.config.ts".cyan()
                );
            }
        },
    }

    Ok(())
}

/// Options for the migration process.
struct MigrationOptions<'a> {
    path: &'a str,
    mode: &'a str,
    confidence_threshold: f64,
    rem_scale: f32,
    custom_theme: &'a [String],
    config_json: Option<&'a String>,
    summary_only: bool,
    extra_css_files: &'a [String],
    inline_css: Option<&'a String>,
    include_tag_selectors: bool,
}

/// Orchestrates the entire migration process for a given path.
///
/// This involves:
/// 1. Resolving configuration from CLI flags and JSON.
/// 2. Scanning for source files and CSS files.
/// 3. Reading CSS contents.
/// 4. Iterating through source files, planning replacements, and optionally writing changes.
/// 5. Generating and printing a comprehensive report.
fn process_migration(opts: MigrationOptions, cli: &Cli) -> anyhow::Result<()> {
    let mut resolved_config = if let Some(json) = opts.config_json {
        serde_json::from_str(json)?
    } else {
        let mut cfg = css2tw_core::Config::default();
        cfg.tailwind.rem_scale = opts.rem_scale;
        cfg.confidence_threshold = opts.confidence_threshold as f32;
        for pair in opts.custom_theme {
            if let Some((k, v)) = pair.split_once('=') {
                cfg.tailwind
                    .custom_theme
                    .insert(k.to_string(), v.to_string());
            }
        }
        cfg
    };

    resolved_config.rewrite.include_tag_selectors = opts.include_tag_selectors;

    // Override config with explicit CLI flags if provided
    if cli.trace {
        resolved_config.agent.include_trace = true;
    }
    if cli.reasons {
        resolved_config.agent.include_reasons = true;
    }
    if cli.pretty {
        resolved_config.agent.compact = false;
    }
    if cli.json {
        // In AI-first CLI, --json defaults to compact unless --pretty is specified
    }

    let is_dry_run = opts.mode != "write";
    let command_name = if opts.mode == "read_only" {
        "scan"
    } else {
        "convert"
    };

    let mut report = css2tw_core::report::Report {
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: command_name.to_string(),
        mode: opts.mode.to_string(),
        summary: css2tw_core::report::Summary {
            files_scanned: 0,
            css_files_scanned: 0,
            source_files_scanned: 0,
            classes_found: 0,
            classes_convertible: 0,
            classes_partially_convertible: 0,
            classes_unconvertible: 0,
            files_changed: 0,
            replacements_planned: 0,
            warnings: 0,
            errors: 0,
        },
        changes: vec![],
        unconverted: vec![],
        warnings: vec![],
        errors: vec![],
    };

    let mut other_files = Vec::new();
    let is_stdin = cli.stdin || opts.path == "-";

    if !is_stdin {
        if let Ok(files) = css2tw_core::source::Scanner::scan_directory(opts.path) {
            other_files = files
                .into_iter()
                .filter(|p| p.extension().and_then(|s| s.to_str()) != Some("css"))
                .collect();
        }
    }

    // CSS files are now ONLY taken from explicit CLI flags
    let mut css_files = Vec::new();
    for extra_css in opts.extra_css_files {
        css_files.push(std::path::PathBuf::from(extra_css));
    }

    report.summary.files_scanned = other_files.len();
    report.summary.css_files_scanned = css_files.len();
    report.summary.source_files_scanned = other_files.len();

    if css_files.is_empty() && opts.inline_css.is_none() {
        report.warnings.push("No CSS files or inline CSS provided. Conversion will rely only on default Tailwind mappings (if any).".to_string());
    }

    let mut css_contents = Vec::new();
    for css_path in &css_files {
        if let Ok(content) = std::fs::read_to_string(css_path) {
            css_contents.push(content);
        }
    }

    // Add inline CSS from CLI
    if let Some(inline) = opts.inline_css {
        css_contents.push(inline.clone());
    }

    let converter = css2tw_core::Converter::new(resolved_config.clone());
    let source_files = if is_stdin {
        use std::io::Read;
        let mut content = String::new();
        std::io::stdin().read_to_string(&mut content)?;
        let extension = cli.stdin_type.clone().unwrap_or_else(|| "html".to_string());
        vec![css2tw_core::source::SourceFile {
            path: format!("stdin.{}", extension),
            content,
        }]
    } else {
        css2tw_core::source::Scanner::read_files_parallel(&other_files)
    };

    if cli.ndjson {
        println!(
            "{}",
            serde_json::json!({
                "type": "start",
                "version": report.version,
                "command": report.command
            })
        );
    }

    for source_file in source_files {
        let replacements = match converter.plan_file(&source_file, &css_contents) {
            Ok(reps) => reps,
            Err(e) => {
                report
                    .errors
                    .push(format!("Error planning {}: {}", source_file.path, e));
                continue;
            }
        };

        if !replacements.is_empty() {
            let mut has_actual_replacements = false;
            let mut change_file = css2tw_core::report::ChangeFile {
                file: source_file.path.clone(),
                status: if is_stdin {
                    "streamed".to_string()
                } else if opts.mode == "read_only" {
                    "found".to_string()
                } else if is_dry_run {
                    "planned".to_string()
                } else {
                    "modified".to_string()
                },
                patches: vec![],
                patched_content: None,
                patch: None,
            };

            for rep in replacements {
                if rep.after.is_empty() {
                    report.summary.classes_unconvertible += 1;
                    report.unconverted.push(css2tw_core::report::Unconverted {
                        selector: rep.before.clone(),
                        reason: rep
                            .failure_reason
                            .unwrap_or(css2tw_core::report::FailureReason::NoMappingFound),
                        details: rep.trace.join("; "),
                        confidence: rep.confidence.score,
                        range: Some(css2tw_core::report::RangeReport {
                            start_byte: rep.span.start,
                            end_byte: rep.span.end,
                        }),
                        raw_css: rep.raw_css.clone(),
                        suggestion: rep.suggestion.clone(),
                        diagnostics: vec![],
                    });
                } else {
                    report.summary.classes_convertible += 1;
                    report.summary.replacements_planned += 1;
                    has_actual_replacements = true;
                    change_file
                        .patches
                        .push(css2tw_core::report::ReplacementReport {
                            range: css2tw_core::report::RangeReport {
                                start_byte: rep.span.start,
                                end_byte: rep.span.end,
                            },
                            before: rep.before.clone(),
                            after: rep.after.clone(),
                            confidence: rep.confidence.clone(),
                            source_selector: "".to_string(),
                            reasons: if resolved_config.agent.include_reasons {
                                rep.reasons.clone()
                            } else {
                                vec![]
                            },
                            trace: if resolved_config.agent.include_trace {
                                rep.trace.clone()
                            } else {
                                vec![]
                            },
                            raw_css: rep.raw_css.clone(),
                            suggestion: rep.suggestion.clone(),
                            diagnostics: vec![],
                        });
                }
            }

            if has_actual_replacements {
                if cli.include_patched || opts.mode == "write" || cli.diff {
                    let actual_replacements: Vec<_> = change_file
                        .patches
                        .iter()
                        .map(|r| css2tw_core::rewrite::patch::Replacement {
                            span: css2tw_core::source::class_usage::Span {
                                start: r.range.start_byte,
                                end: r.range.end_byte,
                            },
                            before: r.before.clone(),
                            after: r.after.clone(),
                            confidence: r.confidence.clone(),
                            reasons: r.reasons.clone(),
                            trace: r.trace.clone(),
                            raw_css: r.raw_css.clone(),
                            suggestion: r.suggestion.clone(),
                            failure_reason: None,
                            diagnostics: r.diagnostics.clone(),
                        })
                        .collect();

                    let patched = css2tw_core::rewrite::patch::apply_patches(
                        &source_file.content,
                        &actual_replacements,
                    );
                    if cli.include_patched {
                        change_file.patched_content = Some(patched.clone());
                    }
                    if cli.diff {
                        let diff_str =
                            generate_diff(&source_file.content, &patched, &source_file.path);
                        if !cli.json && !cli.ndjson {
                            println!("{}", diff_str);
                        }
                        change_file.patch = Some(css2tw_core::report::Patch {
                            format: "unified".to_string(),
                            content: diff_str,
                        });
                    }
                    if opts.mode == "write" && !is_stdin {
                        if let Err(e) = std::fs::write(&source_file.path, patched) {
                            report
                                .errors
                                .push(format!("Failed to write {}: {}", source_file.path, e));
                        } else {
                            report.summary.files_changed += 1;
                        }
                    }
                }

                let is_filtered =
                    !cli.file_only.is_empty() && !cli.file_only.contains(&source_file.path);

                if !is_filtered {
                    if cli.ndjson {
                        println!(
                            "{}",
                            serde_json::json!({
                                "type": "file",
                                "data": change_file
                            })
                        );
                    }
                    report.changes.push(change_file);
                }
            }
        }
    }

    if opts.summary_only {
        report.changes = vec![];
        report.unconverted = vec![];
    }

    if cli.ndjson {
        println!(
            "{}",
            serde_json::json!({
                "type": "summary",
                "data": report.summary
            })
        );
    } else if cli.json {
        print_report(&report, cli)?;
    } else {
        match opts.mode {
            "read_only" => {
                println!("{} analyzed in {}", "Scan".bold().blue(), opts.path.bold());
                println!(
                    "Found {} convertible classes across {} source files.",
                    report.summary.replacements_planned.to_string().green(),
                    report.summary.source_files_scanned.to_string().cyan()
                );
            }
            _ => {
                let status_msg = if opts.mode == "write" {
                    "completed".green()
                } else {
                    "planned (dry-run)".yellow()
                };
                println!(
                    "{} {} for path: {}",
                    "Migration".bold(),
                    status_msg,
                    opts.path.bold()
                );
                println!(
                    "Scanned {} source files. Found {} replacements. Modified {} files.",
                    report.summary.source_files_scanned.to_string().cyan(),
                    report.summary.replacements_planned.to_string().green(),
                    report.summary.files_changed.to_string().yellow()
                );
            }
        }
    }

    Ok(())
}

/// Formats and prints the final JSON report according to CLI flags.
fn print_report(report: &css2tw_core::report::Report, cli: &Cli) -> anyhow::Result<()> {
    if cli.pretty {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
        println!("{}", serde_json::to_string(report)?);
    }
    Ok(())
}

/// Generates a unified diff between two strings.
fn generate_diff(old: &str, new: &str, filename: &str) -> String {
    similar::TextDiff::from_lines(old, new)
        .unified_diff()
        .context_radius(3)
        .header(filename, filename)
        .to_string()
}
