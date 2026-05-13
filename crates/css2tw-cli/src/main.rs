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
                path,
                "read_only",
                0.8, // default threshold
                4.0, // default rem_scale
                &[],
                None,
                *summary_only,
                css_file,
                css_inline.as_ref(),
                *include_tag_selectors,
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
                path,
                mode,
                *confidence_threshold,
                *rem_scale,
                custom_theme,
                config_json.as_ref(),
                *summary_only,
                css_file,
                css_inline.as_ref(),
                *include_tag_selectors,
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
    }

    Ok(())
}

/// Orchestrates the entire migration process for a given path.
///
/// This involves:
/// 1. Resolving configuration from CLI flags and JSON.
/// 2. Scanning for source files and CSS files.
/// 3. Reading CSS contents.
/// 4. Iterating through source files, planning replacements, and optionally writing changes.
/// 5. Generating and printing a comprehensive report.
fn process_migration(
    path: &str,
    mode: &str,
    confidence_threshold: f64,
    rem_scale: f32,
    custom_theme: &[String],
    config_json: Option<&String>,
    summary_only: bool,
    extra_css_files: &[String],
    inline_css: Option<&String>,
    include_tag_selectors: bool,
    cli: &Cli,
) -> anyhow::Result<()> {
    let mut resolved_config = if let Some(json) = config_json {
        serde_json::from_str(json)?
    } else {
        let mut cfg = css2tw_core::Config::default();
        cfg.tailwind.rem_scale = rem_scale;
        cfg.confidence_threshold = confidence_threshold as f32;
        for pair in custom_theme {
            if let Some((k, v)) = pair.split_once('=') {
                cfg.tailwind
                    .custom_theme
                    .insert(k.to_string(), v.to_string());
            }
        }
        cfg
    };

    resolved_config.rewrite.include_tag_selectors = include_tag_selectors;

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

    let is_dry_run = mode != "write";
    let command_name = if mode == "read_only" {
        "scan"
    } else {
        "convert"
    };

    let mut report = css2tw_core::report::Report {
        version: env!("CARGO_PKG_VERSION").to_string(),
        command: command_name.to_string(),
        mode: mode.to_string(),
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
    let is_stdin = cli.stdin || path == "-";

    if !is_stdin {
        if let Ok(files) = css2tw_core::source::Scanner::scan_directory(path) {
            other_files = files
                .into_iter()
                .filter(|p| p.extension().and_then(|s| s.to_str()) != Some("css"))
                .collect();
        }
    }

    // CSS files are now ONLY taken from explicit CLI flags
    let mut css_files = Vec::new();
    for extra_css in extra_css_files {
        css_files.push(std::path::PathBuf::from(extra_css));
    }

    report.summary.files_scanned = other_files.len();
    report.summary.css_files_scanned = css_files.len();
    report.summary.source_files_scanned = other_files.len();

    if css_files.is_empty() && inline_css.is_none() {
        report.warnings.push("No CSS files or inline CSS provided. Conversion will rely only on default Tailwind mappings (if any).".to_string());
    }

    let mut css_contents = Vec::new();
    for css_path in &css_files {
        if let Ok(content) = std::fs::read_to_string(css_path) {
            css_contents.push(content);
        }
    }

    // Add inline CSS from CLI
    if let Some(inline) = inline_css {
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
                } else if mode == "read_only" {
                    "found".to_string()
                } else if is_dry_run {
                    "planned".to_string()
                } else {
                    "modified".to_string()
                },
                replacements: vec![],
                patched_content: None,
            };

            for rep in replacements {
                if rep.after.is_empty() {
                    report.summary.classes_unconvertible += 1;
                    report.unconverted.push(css2tw_core::report::Unconverted {
                        selector: rep.before.clone(),
                        reason: "No mapping found".to_string(),
                        details: rep.trace.join("; "),
                        confidence: rep.confidence.score,
                        range: Some(css2tw_core::report::RangeReport {
                            start_byte: rep.span.start,
                            end_byte: rep.span.end,
                        }),
                        raw_css: rep.raw_css.clone(),
                        suggestion: rep.suggestion.clone(),
                    });
                } else {
                    report.summary.classes_convertible += 1;
                    report.summary.replacements_planned += 1;
                    has_actual_replacements = true;
                    change_file
                        .replacements
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
                        });
                }
            }

            if has_actual_replacements {
                if cli.include_patched || mode == "write" {
                    let actual_replacements: Vec<_> = change_file
                        .replacements
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
                        })
                        .collect();

                    let patched = css2tw_core::rewrite::patch::apply_patches(
                        &source_file.content,
                        &actual_replacements,
                    );
                    if cli.include_patched {
                        change_file.patched_content = Some(patched.clone());
                    }
                    if mode == "write" && !is_stdin {
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

    if summary_only {
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
        match mode {
            "read_only" => {
                println!("{} analyzed in {}", "Scan".bold().blue(), path.bold());
                println!(
                    "Found {} convertible classes across {} source files.",
                    report.summary.replacements_planned.to_string().green(),
                    report.summary.source_files_scanned.to_string().cyan()
                );
            }
            _ => {
                let status_msg = if mode == "write" {
                    "completed".green()
                } else {
                    "planned (dry-run)".yellow()
                };
                println!(
                    "{} {} for path: {}",
                    "Migration".bold(),
                    status_msg,
                    path.bold()
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
