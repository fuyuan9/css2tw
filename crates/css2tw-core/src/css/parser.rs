use crate::error::Css2TwError;
use cssparser::ToCss;
use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use lightningcss::rules::CssRule;
use lightningcss::rules::style::StyleRule;

pub struct ParsedStylesheet<'a> {
    pub ast: StyleSheet<'a, 'a>,
}

pub fn parse_css(css_content: &str) -> Result<ParsedStylesheet<'_>, Css2TwError> {
    let options = ParserOptions::default();
    let ast = StyleSheet::parse(css_content, options)
        .map_err(|e| Css2TwError::Parse(format!("Failed to parse CSS: {:?}", e)))?;
    
    Ok(ParsedStylesheet { ast })
}

/// Extracts simple style rules from the stylesheet.
pub fn extract_style_rules<'i, 'a>(
    stylesheet: &'a ParsedStylesheet<'i>,
) -> Vec<&'a StyleRule<'i>> {
    let mut style_rules = Vec::new();
    
    for rule in &stylesheet.ast.rules.0 {
        if let CssRule::Style(style_rule) = rule {
            style_rules.push(style_rule);
        }
    }
    
    style_rules
}

use std::collections::HashMap;
use lightningcss::properties::Property;
use lightningcss::selector::{Component, PseudoClass};
use crate::tailwind::variant::TailwindVariant;

#[derive(Debug, Clone)]
pub struct TailwindMapping<'i> {
    pub property: Property<'i>,
    pub variant: TailwindVariant,
}

/// Builds a map of CSS class name to its properties and variants
pub fn build_rule_map<'i, 'a>(
    style_rules: &[&'a StyleRule<'i>]
) -> HashMap<String, Vec<TailwindMapping<'i>>> {
    let mut map: HashMap<String, Vec<TailwindMapping>> = HashMap::new();

    for rule in style_rules {
        for selector in &rule.selectors.0 {
            let mut class_name = None;
            let mut variant = TailwindVariant::None;

            // Iterate through components to find the base class and any modifiers
            for component in selector.iter_raw_match_order() {
                match component {
                    Component::Class(name) => {
                        class_name = Some(name.to_string());
                    }
                    Component::NonTSPseudoClass(PseudoClass::Hover) => {
                        variant = TailwindVariant::Hover;
                    }
                    Component::NonTSPseudoClass(PseudoClass::Focus) => {
                        variant = TailwindVariant::Focus;
                    }
                    Component::NonTSPseudoClass(PseudoClass::Active) => {
                        variant = TailwindVariant::Active;
                    }
                    Component::PseudoElement(pe) => {
                        let mut dest = String::new();
                        if pe.to_css(&mut dest).is_ok() {
                            if dest == "::before" || dest == ":before" {
                                variant = TailwindVariant::Before;
                            } else if dest == "::after" || dest == ":after" {
                                variant = TailwindVariant::After;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if let Some(name) = class_name {
                let mappings = rule.declarations.declarations.iter().map(|prop| {
                    TailwindMapping {
                        property: prop.clone(),
                        variant: variant.clone(),
                    }
                }).collect::<Vec<_>>();
                
                map.entry(name).or_default().extend(mappings);
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_css() {
        let css = ".btn { color: red; }";
        let parsed = parse_css(css).unwrap();
        let rules = extract_style_rules(&parsed);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].declarations.declarations.len(), 1);
    }

    #[test]
    fn test_build_rule_map_variants() {
        let css = ".btn:hover { color: green; } .btn::before { content: 'x'; }";
        let parsed = parse_css(css).unwrap();
        let rules = extract_style_rules(&parsed);
        let map = build_rule_map(&rules);

        assert!(map.contains_key("btn"));
        let btn_rules = &map["btn"];
        assert_eq!(btn_rules.len(), 2);

        assert!(btn_rules.iter().any(|m| matches!(m.variant, TailwindVariant::Hover)));
        assert!(btn_rules.iter().any(|m| matches!(m.variant, TailwindVariant::Before)));
    }

    #[test]
    fn test_build_rule_map_multiple_classes() {
        let css = ".a, .b { margin: 0; }";
        let parsed = parse_css(css).unwrap();
        let rules = extract_style_rules(&parsed);
        let map = build_rule_map(&rules);

        assert!(map.contains_key("a"));
        assert!(map.contains_key("b"));
    }
}
