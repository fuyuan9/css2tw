use crate::css::resolver::StyleResolver;
use crate::error::Css2TwError;
use crate::rewrite::patch::Replacement;
use crate::source::{
    class_usage::{ClassUsage, Span},
    ClassUsageParser, SourceFile,
};
use regex::Regex;
use scraper::{Html, Selector};

pub struct HtmlParser;

impl ClassUsageParser for HtmlParser {
    fn extract_classes(&self, source: &SourceFile) -> Result<Vec<ClassUsage>, Css2TwError> {
        let mut classes = Vec::new();
        // Regex to match class="...", class='...', or class=...
        let re = Regex::new(r#"(?i)\bclass\s*=\s*(?:"([^"]*)"|'([^']*)')"#).unwrap();

        for cap in re.captures_iter(&source.content) {
            let match_val = cap.get(1).or_else(|| cap.get(2));
            if let Some(match_val) = match_val {
                let start = match_val.start();
                let class_string = match_val.as_str();

                let mut current_pos = start;
                for part in class_string.split_whitespace() {
                    let part_len = part.len();
                    let part_start =
                        source.content[current_pos..].find(part).unwrap_or(0) + current_pos;
                    let part_end = part_start + part_len;

                    classes.push(ClassUsage {
                        class_name: part.to_string(),
                        span: Span {
                            start: part_start,
                            end: part_end,
                        },
                    });
                    current_pos = part_end;
                }
            }
        }

        Ok(classes)
    }
}

impl HtmlParser {
    pub fn plan_html(
        &self,
        source: &SourceFile,
        style_rules: &[&lightningcss::rules::style::StyleRule],
        rem_scale: f32,
    ) -> Result<Vec<Replacement>, Css2TwError> {
        let mut replacements = Vec::new();
        let html = Html::parse_fragment(&source.content);
        let resolver = StyleResolver::new(style_rules);

        // Track current position in source to handle multiple elements with same classes
        let mut current_search_pos = 0;

        // Iterate through all elements in the document
        for element in html.root_element().select(&Selector::parse("*").unwrap()) {
            let class_attr = element.value().attr("class");
            let tag_name = element.value().name();

            let resolved = resolver.resolve_styles(element);
            let new_classes = resolved.to_tailwind_string(rem_scale);

            if new_classes.is_empty() {
                continue;
            }

            if let Some(class_string) = class_attr {
                // Find this class attribute in the source text
                let search_pattern = format!("class=\"{}\"", class_string);
                let search_pattern_single = format!("class='{}'", class_string);

                let found_pos = source.content[current_search_pos..]
                    .find(&search_pattern)
                    .or_else(|| source.content[current_search_pos..].find(&search_pattern_single));

                if let Some(offset) = found_pos {
                    let start_in_source = current_search_pos + offset;
                    let val_start = start_in_source + 7; // length of 'class="'
                    let val_end = val_start + class_string.len();

                    current_search_pos = val_end;

                    replacements.push(Replacement {
                        span: Span {
                            start: val_start,
                            end: val_end,
                        },
                        before: class_string.to_string(),
                        after: new_classes,
                        confidence: crate::report::ConfidenceReport {
                            score: 1.0,
                            reasons: vec![crate::report::ConfidenceReason::FullMatch],
                        },
                        reasons: vec![],
                        trace: vec!["Matched element in HTML fragment".to_string()],
                    });
                }
            } else {
                // Element has styles but no class attribute. We need to insert one.
                // Find <tag_name
                let tag_pattern = format!("<{}", tag_name);
                if let Some(offset) = source.content[current_search_pos..].find(&tag_pattern) {
                    let tag_start = current_search_pos + offset;
                    let insert_pos = tag_start + tag_pattern.len();

                    current_search_pos = insert_pos;

                    replacements.push(Replacement {
                        span: Span {
                            start: insert_pos,
                            end: insert_pos,
                        },
                        before: "".to_string(),
                        after: format!(" class=\"{}\"", new_classes),
                        confidence: crate::report::ConfidenceReport {
                            score: 1.0,
                            reasons: vec![crate::report::ConfidenceReason::FullMatch],
                        },
                        reasons: vec![],
                        trace: vec!["Injected new class attribute for element".to_string()],
                    });
                }
            }
        }

        Ok(replacements)
    }
}
