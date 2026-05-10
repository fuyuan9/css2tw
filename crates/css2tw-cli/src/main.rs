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

    /// Disable color output
    #[arg(long, global = true)]
    no_color: bool,

    /// Minify JSON output
    #[arg(long, global = true)]
    compact: bool,

    /// Omit conversion trace from output
    #[arg(long, global = true)]
    no_trace: bool,

    /// Omit conversion reasons from output
    #[arg(long, global = true)]
    no_reasons: bool,
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.no_color {
        control::set_override(false);
    }

    match &cli.command {
        Commands::Scan { path, summary_only } => {
            process_migration(
                path,
                "read_only",
                0.8, // default threshold
                4.0, // default rem_scale
                &[],
                None,
                *summary_only,
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
                if cli.compact {
                    println!("{}", serde_json::to_string(&result)?);
                } else {
                    println!("{}", serde_json::to_string_pretty(&result)?);
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
                println!("{}", serde_json::to_string_pretty(&config)?);
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
            
            println!("{}", serde_json::to_string_pretty(&combined)?);
        }
    }

    Ok(())
}

fn process_migration(
    path: &str,
    mode: &str,
    confidence_threshold: f64,
    rem_scale: f32,
    custom_theme: &[String],
    config_json: Option<&String>,
    summary_only: bool,
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
                cfg.tailwind.custom_theme.insert(k.to_string(), v.to_string());
            }
        }
        cfg
    };

    if cli.no_trace {
        resolved_config.agent.include_trace = false;
    }
    if cli.no_reasons {
        resolved_config.agent.include_reasons = false;
    }
    if cli.compact {
        resolved_config.agent.compact = true;
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

    if let Ok(files) = css2tw_core::source::Scanner::scan_directory(path) {
        let (css_files, other_files): (Vec<_>, Vec<_>) = files.into_iter().partition(|p| {
            p.extension().and_then(|s| s.to_str()) == Some("css")
        });

        report.summary.files_scanned = css_files.len() + other_files.len();
        report.summary.css_files_scanned = css_files.len();
        report.summary.source_files_scanned = other_files.len();

        let mut css_contents = Vec::new();
        for css_path in &css_files {
            if let Ok(content) = std::fs::read_to_string(css_path) {
                css_contents.push(content);
            }
        }

        let mut rule_map = std::collections::HashMap::new();
        let mut variable_map = std::collections::HashMap::new();
        let mut stylesheets = Vec::new();
        for content in &css_contents {
            if let Ok(stylesheet) = css2tw_core::css::parser::parse_css(content) {
                stylesheets.push(stylesheet);
            }
        }

        for stylesheet in &stylesheets {
            let rules = css2tw_core::css::parser::extract_style_rules(stylesheet);
            let map = css2tw_core::css::parser::build_rule_map(&rules);
            for (name, mappings) in map {
                rule_map
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .extend(mappings);
            }
            let vars = css2tw_core::css::parser::extract_variables(&rules);
            variable_map.extend(vars);
        }

        let source_files = css2tw_core::source::Scanner::read_files_parallel(&other_files);

        for source_file in source_files {
            use css2tw_core::source::ClassUsageParser;
            let file_path = std::path::Path::new(&source_file.path);
            let replacements = if file_path.extension().and_then(|s| s.to_str()) == Some("html") {
                let html_parser = css2tw_core::source::html::HtmlParser;
                let rules = stylesheets
                    .iter()
                    .flat_map(|s| css2tw_core::css::parser::extract_style_rules(s))
                    .collect::<Vec<_>>();
                html_parser
                    .plan_html(&source_file, &rules, resolved_config.tailwind.rem_scale)
                    .unwrap_or_default()
            } else if file_path.extension().and_then(|s| s.to_str()) == Some("jsx")
                || file_path.extension().and_then(|s| s.to_str()) == Some("tsx")
            {
                let jsx_parser = css2tw_core::source::jsx::JsxParser;
                let rules = stylesheets
                    .iter()
                    .flat_map(|s| css2tw_core::css::parser::extract_style_rules(s))
                    .collect::<Vec<_>>();
                jsx_parser
                    .plan_jsx(&source_file, &rules, resolved_config.tailwind.rem_scale)
                    .unwrap_or_default()
            } else {
                let jsx_parser = css2tw_core::source::jsx::JsxParser;
                let classes = jsx_parser.extract_classes(&source_file).unwrap_or_default();
                css2tw_core::rewrite::planner::ConversionPlanner::plan(
                    &classes,
                    &rule_map,
                    &variable_map,
                    resolved_config.tailwind.rem_scale,
                    resolved_config.confidence_threshold as f64,
                )
                .unwrap_or_default()
            };

            if !replacements.is_empty() {
                report.summary.replacements_planned += replacements.len();

                let mut change_file = css2tw_core::report::ChangeFile {
                    file: source_file.path.clone(),
                    status: if mode == "read_only" {
                        "found".to_string()
                    } else if is_dry_run {
                        "planned".to_string()
                    } else {
                        "modified".to_string()
                    },
                    replacements: vec![],
                };

                for rep in &replacements {
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
                        });
                }

                report.changes.push(change_file);

                if mode == "write" {
                    let patched_content = css2tw_core::rewrite::patch::apply_patches(
                        &source_file.content,
                        &replacements,
                    );
                    if let Err(e) = std::fs::write(&source_file.path, patched_content) {
                        report
                            .errors
                            .push(format!("Failed to write {}: {}", source_file.path, e));
                    } else {
                        report.summary.files_changed += 1;
                    }
                }
            }
        }
    }

    if summary_only {
        report.changes = vec![];
        report.unconverted = vec![];
    }

    if cli.json {
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

fn print_report(report: &css2tw_core::report::Report, cli: &Cli) -> anyhow::Result<()> {
    if cli.compact {
        println!("{}", serde_json::to_string(report)?);
    } else {
        println!("{}", serde_json::to_string_pretty(report)?);
    }
    Ok(())
}
