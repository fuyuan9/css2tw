//! Configuration module for css2tw.
//!
//! Defines the structure and default values for the tool's behavior,
//! including file inclusion/exclusion, Tailwind settings, and AI agent options.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Main configuration structure for css2tw.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "default_threshold")]
    pub confidence_threshold: f32,
    #[serde(default)]
    pub tailwind: TailwindConfig,
    #[serde(default)]
    pub rewrite: RewriteConfig,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default = "default_parsers")]
    pub parsers: std::collections::HashMap<String, ParserType>,
}

fn default_threshold() -> f32 {
    0.9
}

fn default_parsers() -> std::collections::HashMap<String, ParserType> {
    let mut m = std::collections::HashMap::new();
    // HTML parser
    m.insert("html".to_string(), ParserType::Html);
    // JSX / TS parsers
    m.insert("js".to_string(), ParserType::Jsx);
    m.insert("jsx".to_string(), ParserType::Jsx);
    m.insert("ts".to_string(), ParserType::Jsx);
    m.insert("tsx".to_string(), ParserType::Jsx);
    // Fragment parsers for various template engines
    let fragment_exts = [
        "php",
        "phtml",
        "blade.php",
        "jinja",
        "j2",
        "djhtml",
        "erb",
        "html.erb",
        "hbs",
        "handlebars",
        "mustache",
        "gohtml",
        "tmpl",
        "cshtml",
        "vue",
        "svelte",
        "lit",
    ];
    for ext in fragment_exts.iter() {
        m.insert(ext.to_string(), ParserType::Fragment);
    }
    m
}

/// Supported parser types for source file analysis.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ParserType {
    Html,
    Jsx,
    Generic,
    Fragment, // Handles any template fragment (HTML, Blade, Jinja, etc.)
}

/// Configuration for Tailwind CSS generation and theme matching.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TailwindConfig {
    pub version: String,
    pub config_path: Option<String>,
    pub prefer_theme_scale: bool,
    pub allow_arbitrary_values: bool,
    pub rem_scale: f32,
    pub custom_theme: std::collections::HashMap<String, String>,
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            version: "4".to_string(),
            config_path: Some("tailwind.config.js".to_string()),
            prefer_theme_scale: true,
            allow_arbitrary_values: true,
            rem_scale: 4.0,
            custom_theme: std::collections::HashMap::new(),
        }
    }
}

/// Configuration for how source files are rewritten.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewriteConfig {
    pub preserve_unknown_css: bool,
    pub remove_converted_css: bool,
    pub sort_tailwind_classes: bool,
    pub migrate_only_existing_classes: bool,
    #[serde(default)]
    pub include_tag_selectors: bool,
}

impl Default for RewriteConfig {
    fn default() -> Self {
        Self {
            preserve_unknown_css: true,
            remove_converted_css: false,
            sort_tailwind_classes: true,
            migrate_only_existing_classes: true,
            include_tag_selectors: false,
        }
    }
}

/// Configuration for AI agent integration and output optimization.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub deterministic: bool,
    #[serde(default = "default_true")]
    pub compact: bool,
    #[serde(default)]
    pub include_trace: bool,
    #[serde(default)]
    pub include_reasons: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            deterministic: true,
            compact: true,
            include_trace: false,
            include_reasons: false,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            include: vec!["src/**/*.{html,js,jsx,ts,tsx,vue,svelte,css}".to_string()],
            exclude: vec![
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
            ],
            confidence_threshold: 0.9,
            tailwind: TailwindConfig {
                version: "4".to_string(),
                config_path: Some("tailwind.config.js".to_string()),
                prefer_theme_scale: true,
                allow_arbitrary_values: true,
                rem_scale: 4.0,
                custom_theme: std::collections::HashMap::new(),
            },
            rewrite: RewriteConfig {
                preserve_unknown_css: true,
                remove_converted_css: false,
                sort_tailwind_classes: true,
                migrate_only_existing_classes: true,
                include_tag_selectors: false,
            },
            agent: AgentConfig {
                deterministic: true,
                compact: true,
                include_trace: false,
                include_reasons: false,
            },
            parsers: default_parsers(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_json_deserialization() {
        let json = r##"{
            "include": ["**/*"],
            "exclude": [],
            "confidenceThreshold": 0.95,
            "tailwind": {
                "version": "4",
                "configPath": "tailwind.config.js",
                "preferThemeScale": true,
                "allowArbitraryValues": true,
                "remScale": 4.0,
                "customTheme": {
                    "primary": "#ff0000"
                }
            },
            "rewrite": {
                "preserveUnknownCss": true,
                "removeConvertedCss": false,
                "sortTailwindClasses": true,
                "migrateOnlyExistingClasses": true
            },
            "agent": {
                "deterministic": true
            }
        }"##;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.tailwind.custom_theme.get("primary").unwrap(),
            "#ff0000"
        );
        assert_eq!(config.confidence_threshold, 0.95);
    }

    #[test]
    fn test_agent_config_defaults() {
        let json = r##"{
            "agent": {
                "deterministic": true
            }
        }"##;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.agent.include_trace);
        assert!(!config.agent.include_reasons);
        assert!(config.agent.compact);
    }
}
