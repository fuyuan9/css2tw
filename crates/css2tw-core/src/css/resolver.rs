use lightningcss::rules::style::StyleRule;
use scraper::{Selector, ElementRef};
use crate::tailwind::variant::TailwindVariant;
use crate::css::parser::TailwindMapping;

#[derive(Debug, Clone)]
pub struct ResolvedElementStyle<'i> {
    pub properties: Vec<TailwindMapping<'i>>,
    pub variable_map: std::collections::HashMap<String, String>,
}

impl<'i> ResolvedElementStyle<'i> {
    pub fn to_tailwind_string(&self, rem_scale: f32) -> String {
        use crate::tailwind::mapper::map_property;
        use std::collections::{HashMap, HashSet};

        // Group properties by category and variant to resolve conflicts (specificity)
        let mut best_props: HashMap<(String, crate::tailwind::variant::TailwindVariant), &TailwindMapping> = HashMap::new();

        for mapping in &self.properties {
            let mut name = String::new();
            let mut printer = lightningcss::printer::Printer::new(&mut name, Default::default());
            // We use the property name as the key for conflict resolution
            let _ = mapping.property.to_css(&mut printer, false);
            if let Some(pos) = name.find(':') {
                name = name[..pos].to_string();
            }
            
            let key = (name, mapping.variant.clone());
            if let Some(existing) = best_props.get(&key) {
                // If specificity is equal or higher, the later one wins
                if mapping.specificity >= existing.specificity {
                    best_props.insert(key, mapping);
                }
            } else {
                best_props.insert(key, mapping);
            }
        }

        let mut tailwind_classes = Vec::new();
        for mapping in best_props.values() {
            if let Some(tw_class) = map_property(
                &mapping.property,
                &mapping.variant,
                rem_scale,
                &self.variable_map,
            ) {
                tailwind_classes.push(tw_class);
            }
        }
        
        if tailwind_classes.is_empty() {
            return String::new();
        }
        
        let mut unique: Vec<_> = tailwind_classes.into_iter().collect::<HashSet<_>>().into_iter().collect();
        unique.sort();
        unique.join(" ")
    }
}

pub struct StyleResolver<'i, 'a> {
    pub style_rules: &'a [ &'a StyleRule<'i> ],
    pub variable_map: std::collections::HashMap<String, String>,
}

impl<'i, 'a> StyleResolver<'i, 'a> {
    pub fn new(style_rules: &'a [ &'a StyleRule<'i> ]) -> Self {
        let variable_map = crate::css::parser::extract_variables(style_rules);
        Self { style_rules, variable_map }
    }

    pub fn resolve_styles(&self, element: ElementRef) -> ResolvedElementStyle<'i> {
        let mut resolved_props: Vec<TailwindMapping> = Vec::new();

        for rule in self.style_rules {
            for selector in &rule.selectors.0 {
                // Convert lightningcss selector to string for scraper
                let mut sel_str = String::new();
                let mut printer = lightningcss::printer::Printer::new(&mut sel_str, lightningcss::printer::PrinterOptions::default());
                if lightningcss::traits::ToCss::to_css(selector, &mut printer).is_ok() {
                    // Note: scraper doesn't support pseudo-elements in selectors for matching against elements
                    // We need to handle pseudo-elements separately.
                    
                    let clean_sel_str = self.clean_selector(selector);
                    
                    let scraper_sel = Selector::parse(&clean_sel_str);
                    if let Ok(scraper_sel) = scraper_sel {
                        if scraper_sel.matches(&element) {
                            // Determine variant from original selector
                            let variant = self.extract_variant(selector);
                            let specificity = selector.specificity();

                            for prop in &rule.declarations.declarations {
                                let m = TailwindMapping {
                                    property: prop.clone(),
                                    variant: variant.clone(),
                                    specificity,
                                };
                                resolved_props.push(m);
                            }
                        }
                    }
                }
            }
        }

        ResolvedElementStyle {
            properties: resolved_props,
            variable_map: self.variable_map.clone(),
        }
    }

    fn clean_selector(&self, selector: &lightningcss::selector::Selector<'i>) -> String {
        let mut sel_str = String::new();
        let mut printer = lightningcss::printer::Printer::new(&mut sel_str, lightningcss::printer::PrinterOptions::default());
        let _ = lightningcss::traits::ToCss::to_css(selector, &mut printer);
        
        let mut result = String::new();
        let mut depth = 0;
        let mut skip = false;
        for c in sel_str.chars() {
            if c == '(' { depth += 1; }
            if c == ')' { depth -= 1; }
            
            if depth == 0 && c == ':' {
                skip = true;
            }
            if skip && depth == 0 && (c == ' ' || c == '>' || c == '+' || c == '~') {
                skip = false;
            }
            
            if !skip {
                result.push(c);
            }
        }
        
        let res = result.trim();
        if res.is_empty() {
            "*".to_string()
        } else {
            res.to_string()
        }
    }

    fn extract_variant(&self, selector: &lightningcss::selector::Selector<'i>) -> TailwindVariant {
        use cssparser::ToCss;
        let mut variant = TailwindVariant::None;
        
        for component in selector.iter_raw_match_order() {
            let mut dest = String::new();
            if component.to_css(&mut dest).is_ok() {
                match dest.as_str() {
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
                    s if s.starts_with(":nth-child(") => {
                        let val = s.strip_prefix(":nth-child(").unwrap().strip_suffix(')').unwrap();
                        variant = TailwindVariant::Arbitrary(format!("nth-[{}]", val));
                    }
                    "::before" | ":before" => variant = TailwindVariant::Before,
                    "::after" | ":after" => variant = TailwindVariant::After,
                    "::placeholder" | ":placeholder" => variant = TailwindVariant::Placeholder,
                    "::marker" => variant = TailwindVariant::Marker,
                    "::selection" => variant = TailwindVariant::Selection,
                    s if s.starts_with("::") => {
                        let val = s.strip_prefix("::").unwrap();
                        variant = TailwindVariant::Arbitrary(format!("[&::{}]", val));
                    }
                    s if s.starts_with(':') && !s.starts_with("::") => {
                        let val = s.strip_prefix(':').unwrap();
                        variant = TailwindVariant::Arbitrary(format!("[&:{}]", val));
                    }
                    _ => {}
                }
            }
        }
        variant
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lightningcss::stylesheet::{StyleSheet, ParserOptions};
    use lightningcss::properties::Property;
    use scraper::Html;

    #[test]
    fn test_resolve_simple_class() {
        let css = ".card { padding: 10px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        let resolver = StyleResolver::new(&rules);

        let html = Html::parse_fragment("<div class=\"card\"></div>");
        let element = html.root_element().select(&Selector::parse(".card").unwrap()).next().unwrap();

        let resolved = resolver.resolve_styles(element);
        assert!(!resolved.properties.is_empty());
        
        // Should have padding property
        use lightningcss::properties::Property;
        assert!(resolved.properties.iter().any(|m| matches!(m.property, Property::Padding(_))));
    }

    #[test]
    fn test_resolve_tag_specific() {
        let css = "button.btn { color: red; } div.btn { color: blue; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        let resolver = StyleResolver::new(&rules);

        // Test button
        let html_btn = Html::parse_fragment("<button class=\"btn\"></button>");
        let btn = html_btn.root_element().select(&Selector::parse("button").unwrap()).next().unwrap();
        let res_btn = resolver.resolve_styles(btn);
        
        // Should have red color
        let mut dest = String::new();
        res_btn.properties[0].property.to_css(&mut lightningcss::printer::Printer::new(&mut dest, Default::default()), false).unwrap();
        assert!(dest.contains("red") || dest.contains("#f00"));

        // Test div
        let html_div = Html::parse_fragment("<div class=\"btn\"></div>");
        let div = html_div.root_element().select(&Selector::parse("div").unwrap()).next().unwrap();
        let res_div = resolver.resolve_styles(div);
        
        // Should have blue color
        let mut dest = String::new();
        res_div.properties[0].property.to_css(&mut lightningcss::printer::Printer::new(&mut dest, Default::default()), false).unwrap();
        assert!(dest.contains("blue") || dest.contains("#00f"));
    }

    #[test]
    fn test_resolve_pseudo() {
        let css = ".btn:hover { color: green; } .btn::before { padding: 5px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        let resolver = StyleResolver::new(&rules);

        let html = Html::parse_fragment("<button class=\"btn\"></button>");
        let btn = html.root_element().select(&Selector::parse(".btn").unwrap()).next().unwrap();
        let resolved = resolver.resolve_styles(btn);

        assert_eq!(resolved.properties.len(), 2);
        
        let hover_mapping = resolved.properties.iter().find(|m| matches!(m.variant, TailwindVariant::Hover)).unwrap();
        assert!(matches!(hover_mapping.property, Property::Color(_)));

        let before_mapping = resolved.properties.iter().find(|m| matches!(m.variant, TailwindVariant::Before)).unwrap();
        assert!(matches!(before_mapping.property, Property::Padding(_)));
    }

    #[test]
    fn test_resolve_variables() {
        let css = ":root { --main-bg: #ff0000; } .card { background-color: var(--main-bg); }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        let resolver = StyleResolver::new(&rules);

        let html = Html::parse_fragment("<div class=\"card\"></div>");
        let element = html.root_element().select(&Selector::parse(".card").unwrap()).next().unwrap();

        let resolved = resolver.resolve_styles(element);
        let tw = resolved.to_tailwind_string(4.0);
        println!("Variable resolve Output: {}", tw);
        
        // Should resolve to red background
        assert!(tw.contains("bg-[#ff0000]") || tw.contains("bg-[red]"));
    }

    #[test]
    fn test_specificity_optimization() {
        let css = ".card { color: red; } div.card { color: blue; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        let resolver = StyleResolver::new(&rules);

        let html = Html::parse_fragment("<div class=\"card\"></div>");
        let element = html.root_element().select(&Selector::parse(".card").unwrap()).next().unwrap();

        let resolved = resolver.resolve_styles(element);
        let tw = resolved.to_tailwind_string(4.0);
        println!("Specificity Output: {}", tw);
        
        // div.card (specificity 0,1,1) should beat .card (specificity 0,1,0)
        // blue is normalized to #00f by lightningcss
        assert!(tw.contains("#00f"));
        assert!(!tw.contains("red"));
    }
}
