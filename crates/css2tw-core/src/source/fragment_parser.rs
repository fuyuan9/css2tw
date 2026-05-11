use crate::css::resolver::StyleResolver;
use crate::error::Css2TwError;
use crate::rewrite::patch::Replacement;
use crate::source::{
    class_usage::{ClassUsage, Span},
    ClassUsageParser, SourceFile,
};
use regex::Regex;
use scraper::{Html, Selector};

/// Parser for template fragments (PHP, Blade, Jinja2, etc.)
/// Uses `scraper::Html::parse_fragment` to handle partial HTML while preserving template delimiters.
pub struct FragmentParser;

impl ClassUsageParser for FragmentParser {
    fn extract_classes(&self, source: &SourceFile) -> Result<Vec<ClassUsage>, Css2TwError> {
        let mut classes = Vec::new();
        // Robust regex for class/className attributes, handling multiline and varied whitespace
        let re = Regex::new(r#"(?i)\b(?:class|className)\s*=\s*(?:"([^"]*)"|'([^']*)')"#).unwrap();

        for cap in re.captures_iter(&source.content) {
            let match_val = cap.get(1).or_else(|| cap.get(2));
            if let Some(m) = match_val {
                let attr_start = m.start();
                let class_string = m.as_str();
                let mut current_offset = 0;

                for part in class_string.split_whitespace() {
                    // Skip parts that look like template tags to avoid false positives
                    if part.contains("{{")
                        || part.contains("{%")
                        || part.contains("<%")
                        || part.contains("@")
                    {
                        continue;
                    }

                    let part_len = part.len();
                    if let Some(rel_start) = class_string[current_offset..].find(part) {
                        let absolute_start = attr_start + current_offset + rel_start;
                        let absolute_end = absolute_start + part_len;
                        classes.push(ClassUsage {
                            class_name: part.to_string(),
                            span: Span {
                                start: absolute_start,
                                end: absolute_end,
                            },
                        });
                        current_offset += rel_start + part_len;
                    }
                }
            }
        }
        Ok(classes)
    }
}

impl FragmentParser {
    /// Plans conversion for fragment files using a placeholder technique to protect template tags.
    pub fn plan_fragment(
        &self,
        source: &SourceFile,
        style_rules: &[&lightningcss::rules::style::StyleRule],
        rem_scale: f32,
    ) -> Result<Vec<Replacement>, Css2TwError> {
        let mut replacements = Vec::new();

        // 1. Identify and protect template tags
        let mut protected_content = source.content.clone();
        let mut placeholders = Vec::new();

        // Common template tag patterns
        let tpl_patterns = [
            r"<\?php.*?\?>",    // PHP
            r"<\?=.*?\?>",      // PHP echo
            r"\{\{.*?\}\}",     // Blade/Jinja/Mustache echo
            r"\{%.*?%\}",       // Jinja/Twig tags
            r"\{#.*?#\}",       // Jinja/Twig comments
            r"\{\!\!.*?\!\!\}", // Blade raw echo
            r"@\w+\(.*?\)",     // Blade directives like @if(...)
            r"@\w+",            // Blade directives like @else, @endif
        ];

        for pattern in tpl_patterns {
            let re = Regex::new(&format!(r"(?s){}", pattern)).unwrap();
            let mut offset = 0;
            while let Some(mat) = re.find(&protected_content[offset..]) {
                let start = offset + mat.start();
                let end = offset + mat.end();
                let original = &protected_content[start..end];
                let placeholder = format!("___CSS2TW_TPL_{}___", placeholders.len());
                placeholders.push(original.to_string());

                protected_content.replace_range(start..end, &placeholder);
                offset = start + placeholder.len();
            }
        }

        // 2. Parse the protected content with scraper
        let fragment = Html::parse_fragment(&protected_content);
        let resolver = StyleResolver::new(style_rules);
        let mut current_search_pos = 0usize;

        let root = fragment.root_element();
        let mut elements = vec![root];
        elements.extend(root.select(&Selector::parse("*").unwrap()));

        for element in elements {
            let class_attr = element.value().attr("class");
            let tag_name = element.value().name();

            // Resolve styles
            let resolved = resolver.resolve_styles(element);
            let tailwind_classes = resolved.to_tailwind_string(rem_scale);

            if tailwind_classes.is_empty() && class_attr.is_none() {
                continue;
            }

            if let Some(class_string) = class_attr {
                // Search for this class attribute in the PROTECTED content
                let escaped_class = regex::escape(class_string);
                let attr_re = Regex::new(&format!(
                    r#"(?i)\b(?:class|className)\s*=\s*(?:"{}"|'{}')"#,
                    escaped_class, escaped_class
                ))
                .unwrap();

                if let Some(mat) = attr_re.find(&protected_content[current_search_pos..]) {
                    let match_start_in_protected = current_search_pos + mat.start();
                    let match_end_in_protected = current_search_pos + mat.end();

                    // Find the value range
                    let val_re = Regex::new(&format!(
                        r#"(?:"({})"|'({})')"#,
                        escaped_class, escaped_class
                    ))
                    .unwrap();
                    if let Some(val_mat) = val_re.captures(
                        &protected_content[match_start_in_protected..match_end_in_protected],
                    ) {
                        let cap = val_mat.get(1).or_else(|| val_mat.get(2)).unwrap();
                        let _val_start_in_protected = match_start_in_protected + cap.start();
                        let val_end_in_protected = match_start_in_protected + cap.end();

                        current_search_pos = val_end_in_protected;

                        // Now we need to map this back to the ORIGINAL content
                        // Since placeholders might have different lengths than original tags,
                        // we need to be careful.

                        // Let's reconstruct the "after" string by keeping unknown classes and template tags
                        let mut final_parts = Vec::new();
                        if !tailwind_classes.is_empty() {
                            final_parts.push(tailwind_classes);
                        }

                        for part in class_string.split_whitespace() {
                            if part.starts_with("___CSS2TW_TPL_") && part.ends_with("___") {
                                // Restore the template tag
                                if let Ok(idx) = part[14..part.len() - 3].parse::<usize>() {
                                    if let Some(original) = placeholders.get(idx) {
                                        final_parts.push(original.clone());
                                    }
                                }
                            } else {
                                // If it's a regular class, we only keep it if it wasn't converted.
                                // In this simplified structural parser, we assume any class that matched
                                // a CSS rule is now represented in tailwind_classes.
                                // However, we don't know which ones those are.
                                // For safety in fragments, let's keep all original classes that were NOT in the CSS.
                                // (This is a bit redundant but safer).
                                let was_matched = style_rules.iter().any(|r| {
                                    r.selectors.0.iter().any(|s| {
                                        s.iter().any(|comp| match comp {
                                            lightningcss::selector::Component::Class(c) => {
                                                c.0.as_ref() == part
                                            }
                                            _ => false,
                                        })
                                    })
                                });
                                if !was_matched {
                                    final_parts.push(part.to_string());
                                }
                            }
                        }

                        let after_string = final_parts.join(" ");
                        if after_string == class_string {
                            continue;
                        }

                        // We need the original span. This is the hardest part.
                        // For now, let's just use the position found in the original source
                        // by searching for the reconstructed class_string.
                        // (Wait, the class_string in scraper ALREADY has placeholders).

                        // Let's find the original class string in the original source.
                        // We can reconstruct what the class string looked like in original source.
                        let mut original_class_string = class_string.to_string();
                        for (i, original) in placeholders.iter().enumerate() {
                            let placeholder = format!("___CSS2TW_TPL_{}___", i);
                            original_class_string =
                                original_class_string.replace(&placeholder, original);
                        }

                        let search_re = Regex::new(&format!(
                            r#"(?i)\b(?:class|className)\s*=\s*(?:"{}"|'{}')"#,
                            regex::escape(&original_class_string),
                            regex::escape(&original_class_string)
                        ))
                        .unwrap();

                        if let Some(orig_mat) = search_re.find(&source.content) {
                            let val_re = Regex::new(&format!(
                                r#"(?:"({})"|'({})')"#,
                                regex::escape(&original_class_string),
                                regex::escape(&original_class_string)
                            ))
                            .unwrap();
                            if let Some(orig_val_mat) = val_re.captures(orig_mat.as_str()) {
                                let cap =
                                    orig_val_mat.get(1).or_else(|| orig_val_mat.get(2)).unwrap();
                                let final_start = orig_mat.start() + cap.start();
                                let final_end = orig_mat.start() + cap.end();

                                replacements.push(Replacement {
                                    span: Span {
                                        start: final_start,
                                        end: final_end,
                                    },
                                    before: original_class_string.clone(),
                                    after: after_string,
                                    confidence: crate::report::ConfidenceReport {
                                        score: 1.0,
                                        reasons: vec![crate::report::ConfidenceReason::FullMatch],
                                    },
                                    reasons: vec![],
                                    trace: vec![
                                        "Matched fragment element with template tag protection"
                                            .to_string(),
                                    ],
                                });
                            }
                        }
                    }
                }
            } else if !tailwind_classes.is_empty() {
                // Insert case
                let tag_pattern = format!("<{}", tag_name);
                let mut search_start = current_search_pos;
                while let Some(offset) = protected_content[search_start..].find(&tag_pattern) {
                    let tag_start_in_protected = search_start + offset;
                    let tag_end_in_protected = tag_start_in_protected + tag_pattern.len();

                    let is_valid = protected_content[tag_end_in_protected..]
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_whitespace() || c == '>' || c == '/');

                    if is_valid {
                        // Map back to original source
                        // This is tricky if there are placeholders before tag_end.
                        // But usually tags themselves don't contain our placeholders (unless it's a dynamic tag).

                        // Let's just find the same tag in the original source.
                        if let Some(orig_offset) = source.content.find(&tag_pattern) {
                            let insert_pos = orig_offset + tag_pattern.len();
                            current_search_pos = tag_end_in_protected;

                            replacements.push(Replacement {
                                span: Span {
                                    start: insert_pos,
                                    end: insert_pos,
                                },
                                before: "".to_string(),
                                after: format!(" class=\"{}\"", tailwind_classes),
                                confidence: crate::report::ConfidenceReport {
                                    score: 1.0,
                                    reasons: vec![crate::report::ConfidenceReason::FullMatch],
                                },
                                reasons: vec![],
                                trace: vec!["Inserted new class attribute in fragment".to_string()],
                            });
                            break;
                        }
                    }
                    search_start = tag_end_in_protected;
                }
            }
        }
        Ok(replacements)
    }
}
