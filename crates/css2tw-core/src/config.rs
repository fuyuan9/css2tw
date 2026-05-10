use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub confidence_threshold: f32,
    pub tailwind: TailwindConfig,
    pub rewrite: RewriteConfig,
    pub agent: AgentConfig,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RewriteConfig {
    pub preserve_unknown_css: bool,
    pub remove_converted_css: bool,
    pub sort_tailwind_classes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub json_only: bool,
    pub deterministic: bool,
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
            },
            agent: AgentConfig {
                json_only: true,
                deterministic: true,
            },
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
                "sortTailwindClasses": true
            },
            "agent": {
                "jsonOnly": true,
                "deterministic": true
            }
        }"##;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.tailwind.custom_theme.get("primary").unwrap(), "#ff0000");
        assert_eq!(config.confidence_threshold, 0.95);
    }
}
