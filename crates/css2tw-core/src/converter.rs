use crate::config::{Config, ParserType};
use crate::css::parser::{build_rule_map, extract_style_rules, parse_css};
use crate::error::Css2TwError;
use crate::rewrite::patch::apply_patches;
use crate::rewrite::planner::ConversionPlanner;
use crate::source::generic::GenericRegexParser;
use crate::source::html::HtmlParser;
use crate::source::jsx::JsxParser;
use crate::source::{ClassUsageParser, SourceFile};
use std::collections::HashMap;

/// The main entry point for CSS to Tailwind conversion.
///
/// It holds the configuration and provides methods to process source files.
pub struct Converter {
    config: Config,
}

impl Converter {
    /// Creates a new Converter instance with the given configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Converts a single source file using the provided CSS contents.
    ///
    /// This method parses the CSS, analyzes the source file for class usages,
    /// determines the best Tailwind class replacements, and applies them to the source content.
    pub fn convert_file(
        &self,
        source: &SourceFile,
        css_contents: &[String],
    ) -> Result<String, Css2TwError> {
        let mut stylesheets = Vec::new();
        for content in css_contents {
            if let Ok(stylesheet) = parse_css(content) {
                stylesheets.push(stylesheet);
            }
        }

        let mut rule_map = HashMap::new();
        let mut variable_map = HashMap::new();
        for stylesheet in &stylesheets {
            let rules = extract_style_rules(stylesheet);
            let map = build_rule_map(&rules);
            for (name, mappings) in map {
                rule_map
                    .entry(name)
                    .or_insert_with(Vec::new)
                    .extend(mappings);
            }
            let vars = crate::css::parser::extract_variables(&rules);
            variable_map.extend(vars);
        }

        let path = std::path::Path::new(&source.path);
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let parser_type = self
            .config
            .parsers
            .get(extension)
            .cloned()
            .unwrap_or(ParserType::Generic);

        let replacements = match parser_type {
            ParserType::Html => {
                let html_parser = HtmlParser;
                let rules = stylesheets
                    .iter()
                    .flat_map(|s| extract_style_rules(s))
                    .collect::<Vec<_>>();
                html_parser
                    .plan_html(source, &rules, self.config.tailwind.rem_scale)
                    .unwrap_or_default()
            }
            ParserType::Jsx => {
                let jsx_parser = JsxParser;
                let rules = stylesheets
                    .iter()
                    .flat_map(|s| extract_style_rules(s))
                    .collect::<Vec<_>>();
                jsx_parser
                    .plan_jsx(source, &rules, self.config.tailwind.rem_scale)
                    .unwrap_or_default()
            }
            ParserType::Generic => {
                let generic_parser = GenericRegexParser;
                let classes = generic_parser.extract_classes(source).unwrap_or_default();
                ConversionPlanner::plan(
                    &classes,
                    &rule_map,
                    &variable_map,
                    self.config.tailwind.rem_scale,
                    self.config.confidence_threshold as f64,
                )
                .unwrap_or_default()
            }
        };

        Ok(apply_patches(&source.content, &replacements))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::SourceFile;

    #[test]
    fn test_pure_conversion_with_trace() {
        let mut config = Config::default();
        config.tailwind.rem_scale = 4.0;

        let converter = Converter::new(config);

        let source = SourceFile {
            path: "test.html".to_string(),
            content: r#"<div class="pt-16"></div>"#.to_string(),
        };

        let css = r#".pt-16 { padding-top: 16px; }"#.to_string();

        let result = converter.convert_file(&source, &[css]).unwrap();

        // Check if converted correctly (assuming resolver works)
        assert!(result.contains("pt-4"));
        assert!(!result.contains("pt-16"));
    }

    #[test]
    fn test_custom_theme_injection() {
        let mut config = Config::default();
        config
            .tailwind
            .custom_theme
            .insert("brand-primary".to_string(), "#ff0000".to_string());

        let converter = Converter::new(config);

        // This is a simplified test, real matching depends on tailwind module implementation
        // But we are testing if the converter can run with custom config
        let source = SourceFile {
            path: "test.jsx".to_string(),
            content: r#"<div className="btn"></div>"#.to_string(),
        };

        let css = r#".btn { color: #ff0000; }"#.to_string();
        let _result = converter.convert_file(&source, &[css]).unwrap();
    }
}
