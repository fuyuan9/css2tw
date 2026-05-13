use crate::css::resolver::StyleResolver;
use crate::error::Css2TwError;
use crate::rewrite::patch::Replacement;
use crate::source::{
    class_usage::{ClassUsage, Span},
    ClassUsageParser, SourceFile,
};
use regex::Regex;
use scraper::{Html, Selector};

/// Specialized parser for HTML files.
/// It uses a combination of regex for class extraction and a full HTML parser for complex mapping.
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
    /// Plans the conversion of an HTML file by analyzing its full structure.
    /// This allows resolving tag selectors and parent-child relationships.
    pub fn plan_html(
        &self,
        source: &SourceFile,
        style_rules: &[&lightningcss::rules::style::StyleRule],
        rem_scale: f32,
        migrate_only_existing_classes: bool,
        include_tag_selectors: bool,
    ) -> Result<Vec<Replacement>, Css2TwError> {
        let mut replacements = Vec::new();
        let html = Html::parse_document(&source.content);
        let resolver = StyleResolver::new(style_rules, include_tag_selectors);

        // Track current position in source to handle multiple elements with same classes
        let mut current_search_pos = 0;

        // Iterate through all elements in the document, including the root element (html)
        let root = html.root_element();
        let mut elements = vec![root];
        elements.extend(root.select(&Selector::parse("*").unwrap()));

        for element in elements {
            let class_attr = element.value().attr("class");

            let resolved = resolver.resolve_styles(element);
            let raw_css = resolved.get_raw_css();
            let new_classes = resolved.to_tailwind_string(rem_scale);

            if new_classes.is_empty() && raw_css.is_none() {
                continue;
            }

            if migrate_only_existing_classes && class_attr.is_none() {
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
                    let val_start = if source.content[start_in_source..].starts_with("class=\"") {
                        start_in_source + 7
                    } else {
                        start_in_source + 7 // length of 'class='
                    };
                    let val_end = val_start + class_string.len();

                    current_search_pos = val_end;

                    replacements.push(resolved.create_replacement(
                        Span {
                            start: val_start,
                            end: val_end,
                        },
                        class_string.to_string(),
                        rem_scale,
                        vec!["Matched element in HTML document".to_string()],
                    ));
                }
            } else if !new_classes.is_empty() {
                // Element has styles but no class attribute. We need to insert one.
                let tag_pattern = format!("<{}", element.value().name());
                let mut search_start = current_search_pos;

                while let Some(offset) = source.content[search_start..].find(&tag_pattern) {
                    let tag_start = search_start + offset;
                    let tag_end = tag_start + tag_pattern.len();

                    // Verify it's a complete tag name
                    let is_valid_tag = source.content[tag_end..]
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_whitespace() || c == '>' || c == '/');

                    if is_valid_tag {
                        let insert_pos = tag_end;
                        current_search_pos = insert_pos;

                        let mut rep = resolved.create_replacement(
                            Span {
                                start: insert_pos,
                                end: insert_pos,
                            },
                            "".to_string(),
                            rem_scale,
                            vec!["Injected new class attribute for element".to_string()],
                        );
                        rep.after = format!(" class=\"{}\"", new_classes);
                        replacements.push(rep);
                        break;
                    }
                    search_start = tag_end;
                }
            }
        }

        Ok(replacements)
    }
}
