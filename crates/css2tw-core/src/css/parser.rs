use crate::error::Css2TwError;
use crate::tailwind::variant::TailwindVariant;
use lightningcss::printer::{Printer, PrinterOptions};
use lightningcss::properties::Property;
use lightningcss::rules::style::StyleRule;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::traits::ToCss;
use std::collections::HashMap;

/// A wrapper around a lightningcss StyleSheet.
pub struct ParsedStylesheet<'a> {
    pub ast: StyleSheet<'a, 'a>,
}

/// Parses a CSS string into a lightningcss AST.
pub fn parse_css(css_content: &str) -> Result<ParsedStylesheet<'_>, Css2TwError> {
    let options = ParserOptions::default();
    let ast = StyleSheet::parse(css_content, options)
        .map_err(|e| Css2TwError::Parse(format!("Failed to parse CSS: {:?}", e)))?;

    Ok(ParsedStylesheet { ast })
}

/// A rule accompanied by any variants inherited from parent blocks (like @media).
pub struct RuleWithContext<'a, 'i> {
    pub rule: &'a StyleRule<'i>,
    pub context_variants: Vec<TailwindVariant>,
}

/// Extracts style rules from the stylesheet, including those nested in @media blocks.
pub fn extract_style_rules<'i, 'a>(
    stylesheet: &'a ParsedStylesheet<'i>,
) -> Vec<RuleWithContext<'a, 'i>> {
    extract_rules_recursive(&stylesheet.ast.rules.0, Vec::new())
}

fn extract_rules_recursive<'i, 'a>(
    rules: &'a [CssRule<'i>],
    current_variants: Vec<TailwindVariant>,
) -> Vec<RuleWithContext<'a, 'i>> {
    let mut result = Vec::new();

    for rule in rules {
        match rule {
            CssRule::Style(style_rule) => {
                result.push(RuleWithContext {
                    rule: style_rule,
                    context_variants: current_variants.clone(),
                });
            }
            CssRule::Media(media_rule) => {
                let mut next_variants = current_variants.clone();
                let mut media_str = String::new();
                {
                    let mut printer = Printer::new(&mut media_str, PrinterOptions::default());
                    let _ = media_rule.query.to_css(&mut printer);
                }

                let variant = map_media_query(&media_str);
                next_variants.push(variant);

                result.extend(extract_rules_recursive(&media_rule.rules.0, next_variants));
            }
            _ => {}
        }
    }

    result
}

fn map_media_query(query: &str) -> TailwindVariant {
    // Basic mapping for common Tailwind breakpoints
    // Support both traditional (min-width: ...) and modern (width >= ...) syntax
    match query {
        "(min-width: 640px)" | "(width >= 640px)" => TailwindVariant::Media("sm".to_string()),
        "(min-width: 768px)" | "(width >= 768px)" => TailwindVariant::Media("md".to_string()),
        "(min-width: 1024px)" | "(width >= 1024px)" => TailwindVariant::Media("lg".to_string()),
        "(min-width: 1280px)" | "(width >= 1280px)" => TailwindVariant::Media("xl".to_string()),
        "(min-width: 1536px)" | "(width >= 1536px)" => TailwindVariant::Media("2xl".to_string()),
        _ => {
            // Handle max-width or <= syntax
            if query.contains("max-width:") || query.contains("<=") {
                let re_max = regex::Regex::new(r"(?:max-width:\s*|width\s*<=\s*)([^)]+)").unwrap();
                if let Some(cap) = re_max.captures(query) {
                    let val = cap.get(1).unwrap().as_str().trim();
                    return TailwindVariant::Media(format!("max-[{}]", val));
                }
            }
            // Handle min-width or >= syntax for arbitrary values
            if query.contains("min-width:") || query.contains(">=") {
                let re_min = regex::Regex::new(r"(?:min-width:\s*|width\s*>=\s*)([^)]+)").unwrap();
                if let Some(cap) = re_min.captures(query) {
                    let val = cap.get(1).unwrap().as_str().trim();
                    return TailwindVariant::Media(val.to_string());
                }
            }
            // Fallback to arbitrary variant
            TailwindVariant::Arbitrary(format!("[@media_{}]", query.replace(' ', "_")))
        }
    }
}

/// Extracts CSS custom properties (variables) from style rules.
pub fn extract_variables(style_rules: &[RuleWithContext]) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for rule_ctx in style_rules {
        for decl in &rule_ctx.rule.declarations.declarations {
            if let Property::Custom(_) = decl {
                let mut full = String::new();
                {
                    let mut printer = Printer::new(&mut full, PrinterOptions::default());
                    let _ = decl.to_css(&mut printer, false);
                }
                if let Some(pos) = full.find(':') {
                    let name = full[..pos].trim().to_string();
                    let value = full[pos + 1..].trim().trim_end_matches(';').to_string();
                    map.insert(name, value);
                }
            }
        }
    }
    map
}

/// Represents a mapping from a CSS property to a potential Tailwind class,
/// including any variants (like hover:, md:) and the original selector's specificity.
#[derive(Debug, Clone)]
pub struct TailwindMapping<'i> {
    pub property: Property<'i>,
    pub variants: Vec<TailwindVariant>,
    pub specificity: u32,
    pub important: bool,
}

/// Builds a map of CSS class name to its properties and variants
pub fn build_rule_map<'i, 'a>(
    style_rules: &'a [RuleWithContext<'a, 'i>],
) -> HashMap<String, Vec<TailwindMapping<'i>>> {
    let mut map: HashMap<String, Vec<TailwindMapping<'i>>> = HashMap::new();

    for rule_ctx in style_rules {
        let rule = rule_ctx.rule;
        let mut class_name = None;
        let mut variants = rule_ctx.context_variants.clone();

        // Currently we only support single-selector rules for simplicity
        if rule.selectors.0.is_empty() {
            continue;
        }
        let selector = &rule.selectors.0[0];
        let mut sel_str = String::new();
        {
            let mut printer = Printer::new(&mut sel_str, PrinterOptions::default());
            let _ = ToCss::to_css(selector, &mut printer);
        }

        if let Some(dot_pos) = sel_str.find('.') {
            let from_dot = &sel_str[dot_pos..];
            let (base, _) = if let Some(colon_pos) = from_dot.find(':') {
                let (c, p) = from_dot.split_at(colon_pos);
                // Map CSS pseudo-classes/elements to Tailwind variants
                match p {
                    ":hover" => variants.push(TailwindVariant::Hover),
                    ":focus" => variants.push(TailwindVariant::Focus),
                    ":active" => variants.push(TailwindVariant::Active),
                    ":disabled" => variants.push(TailwindVariant::Disabled),
                    ":checked" => variants.push(TailwindVariant::Checked),
                    ":valid" => variants.push(TailwindVariant::Valid),
                    ":invalid" => variants.push(TailwindVariant::Invalid),
                    ":required" => variants.push(TailwindVariant::Required),
                    ":first-child" => variants.push(TailwindVariant::First),
                    ":last-child" => variants.push(TailwindVariant::Last),
                    ":nth-child(odd)" | ":nth-child(2n+1)" => variants.push(TailwindVariant::Odd),
                    ":nth-child(even)" | ":nth-child(2n)" => variants.push(TailwindVariant::Even),
                    "::before" | ":before" => variants.push(TailwindVariant::Before),
                    "::after" | ":after" => variants.push(TailwindVariant::After),
                    "::placeholder" | ":placeholder" => variants.push(TailwindVariant::Placeholder),
                    "::marker" => variants.push(TailwindVariant::Marker),
                    "::selection" => variants.push(TailwindVariant::Selection),
                    _ if p.starts_with(":nth-child(") => {
                        if let Some(val) = p
                            .strip_prefix(":nth-child(")
                            .and_then(|s| s.strip_suffix(')'))
                        {
                            variants.push(TailwindVariant::Arbitrary(format!("nth-[{}]", val)));
                        }
                    }
                    _ if p.starts_with("::") => {
                        if let Some(stripped) = p.strip_prefix("::") {
                            variants.push(TailwindVariant::Arbitrary(format!("[&::{}]", stripped)));
                        }
                    }
                    _ if p.starts_with(':') => {
                        if let Some(stripped) = p.strip_prefix(':') {
                            variants.push(TailwindVariant::Arbitrary(format!("[&:{}]", stripped)));
                        }
                    }
                    _ => {}
                }
                (c, Some(p))
            } else {
                (from_dot, None)
            };

            // base is something like ".my-class"
            if let Some(stripped) = base.strip_prefix('.') {
                // Ensure we don't have further dots or other chars
                let end_pos = stripped.find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
                let final_name = if let Some(ep) = end_pos {
                    &stripped[..ep]
                } else {
                    stripped
                };
                if !final_name.is_empty() {
                    class_name = Some(final_name.to_string());
                }
            }
        }

        if let Some(name) = class_name {
            let specificity = selector.specificity();

            // Handle normal declarations
            let mappings = rule
                .declarations
                .declarations
                .iter()
                .map(|prop| TailwindMapping {
                    property: prop.clone(),
                    variants: variants.clone(),
                    specificity,
                    important: false,
                });

            // Handle important declarations
            let important_mappings =
                rule.declarations
                    .important_declarations
                    .iter()
                    .map(|prop| TailwindMapping {
                        property: prop.clone(),
                        variants: variants.clone(),
                        specificity,
                        important: true,
                    });

            map.entry(name)
                .or_default()
                .extend(mappings.chain(important_mappings));
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use lightningcss::stylesheet::{ParserOptions, StyleSheet};

    #[test]
    fn test_extract_variables() {
        let css = ":root { --main-color: #ff0000; --secondary: 1rem; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = ParsedStylesheet { ast: stylesheet };
        let rules = extract_style_rules(&parsed);
        let vars = extract_variables(&rules);

        assert_eq!(vars.get("--main-color").unwrap(), "red");
        assert_eq!(vars.get("--secondary").unwrap(), "1rem");
    }

    #[test]
    fn test_build_rule_map_robustness() {
        let css = "
            div { color: red; }
            .btn:hover { color: blue; }
            input[type='text'] { color: green; }
            div.active:nth-child(2n) { color: yellow; }
        ";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = ParsedStylesheet { ast: stylesheet };
        let rules = extract_style_rules(&parsed);
        let map = build_rule_map(&rules);

        // div should be skipped (no class)
        assert!(!map.contains_key("div"));

        // .btn:hover should be present
        assert!(map.contains_key("btn"));

        // div.active:nth-child(2n) should be present as "active"
        assert!(map.contains_key("active"));

        // Test with a pseudo-class that has no closing paren but is still valid CSS (lexically)
        let css_partial = ".broken:nth-child(2n { color: red; }";
        if let Ok(ast) = StyleSheet::parse(css_partial, ParserOptions::default()) {
            let parsed = ParsedStylesheet { ast };
            let rules = extract_style_rules(&parsed);
            let _map = build_rule_map(&rules);
            // Should not panic
        }
    }

    #[test]
    fn test_media_query_extraction() {
        let css = "
            @media (max-width: 1120px) {
                .test { color: red; }
            }
            @media (min-width: 768px) {
                .test:hover { color: blue; }
            }
        ";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = ParsedStylesheet { ast: stylesheet };
        let rules = extract_style_rules(&parsed);
        let map = build_rule_map(&rules);

        assert!(map.contains_key("test"));
        let mappings = map.get("test").unwrap();

        // Check for max-width: 1120px
        assert!(mappings.iter().any(|m| m
            .variants
            .contains(&TailwindVariant::Media("max-[1120px]".to_string()))));

        // Check for md:hover (min-width: 768px + hover)
        assert!(mappings.iter().any(|m| {
            m.variants
                .contains(&TailwindVariant::Media("md".to_string()))
                && m.variants.contains(&TailwindVariant::Hover)
        }));
    }
}
