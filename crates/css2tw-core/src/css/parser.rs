use crate::error::Css2TwError;
use crate::tailwind::variant::TailwindVariant;
use cssparser::ToCss;
use lightningcss::printer::{Printer, PrinterOptions};
use lightningcss::properties::Property;
use lightningcss::rules::style::StyleRule;
use lightningcss::rules::CssRule;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
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

/// Extracts simple style rules from the stylesheet.
pub fn extract_style_rules<'i, 'a>(stylesheet: &'a ParsedStylesheet<'i>) -> Vec<&'a StyleRule<'i>> {
    let mut style_rules = Vec::new();

    for rule in &stylesheet.ast.rules.0 {
        if let CssRule::Style(style_rule) = rule {
            style_rules.push(style_rule);
        }
    }

    style_rules
}

/// Extracts CSS custom properties (variables) from style rules.
pub fn extract_variables(style_rules: &[&StyleRule]) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for rule in style_rules {
        for decl in &rule.declarations.declarations {
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
/// including any variants (like hover:) and the original selector's specificity.
#[derive(Debug, Clone)]
pub struct TailwindMapping<'i> {
    pub property: Property<'i>,
    pub variant: TailwindVariant,
    pub specificity: u32,
}

/// Builds a map of CSS class name to its properties and variants
pub fn build_rule_map<'i, 'a>(
    style_rules: &'a [&'a StyleRule<'i>],
) -> HashMap<String, Vec<TailwindMapping<'i>>> {
    let mut map: HashMap<String, Vec<TailwindMapping<'i>>> = HashMap::new();

    for rule in style_rules {
        let mut class_name = None;
        let mut variant = TailwindVariant::default();

        // Currently we only support single-selector rules for simplicity
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
                    ":hover" => variant = TailwindVariant::Hover,
                    ":focus" => variant = TailwindVariant::Focus,
                    ":active" => variant = TailwindVariant::Active,
                    ":disabled" => variant = TailwindVariant::Disabled,
                    ":checked" => variant = TailwindVariant::Checked,
                    ":valid" => variant = TailwindVariant::Valid,
                    ":invalid" => variant = TailwindVariant::Invalid,
                    ":required" => variant = TailwindVariant::Required,
                    ":first-child" => variant = TailwindVariant::First,
                    ":last-child" => variant = TailwindVariant::Last,
                    ":nth-child(odd)" | ":nth-child(2n+1)" => variant = TailwindVariant::Odd,
                    ":nth-child(even)" | ":nth-child(2n)" => variant = TailwindVariant::Even,
                    "::before" | ":before" => variant = TailwindVariant::Before,
                    "::after" | ":after" => variant = TailwindVariant::After,
                    "::placeholder" | ":placeholder" => variant = TailwindVariant::Placeholder,
                    "::marker" => variant = TailwindVariant::Marker,
                    "::selection" => variant = TailwindVariant::Selection,
                    _ if p.starts_with(":nth-child(") => {
                        if let Some(val) = p
                            .strip_prefix(":nth-child(")
                            .and_then(|s| s.strip_suffix(')'))
                        {
                            variant = TailwindVariant::Arbitrary(format!("nth-[{}]", val));
                        }
                    }
                    _ if p.starts_with("::") => {
                        variant = TailwindVariant::Arbitrary(format!("[&{}]", p));
                    }
                    _ if p.starts_with(':') => {
                        variant = TailwindVariant::Arbitrary(format!("[&{}]", p));
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
            let mappings = rule
                .declarations
                .declarations
                .iter()
                .map(|prop| TailwindMapping {
                    property: prop.clone(),
                    variant: variant.clone(),
                    specificity,
                })
                .collect::<Vec<_>>();

            map.entry(name).or_default().extend(mappings);
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
        // Note: StyleSheet::parse might fail on very broken CSS, so we test semantic robustness
        let css_partial = ".broken:nth-child(2n { color: red; }";
        if let Ok(ast) = StyleSheet::parse(css_partial, ParserOptions::default()) {
            let parsed = ParsedStylesheet { ast };
            let rules = extract_style_rules(&parsed);
            let _map = build_rule_map(&rules);
            // Should not panic
        }
    }
}
