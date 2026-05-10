use lightningcss::properties::Property;
use lightningcss::properties::display::{Display, DisplayKeyword, DisplayOutside, DisplayInside, DisplayPair};
use lightningcss::properties::position::Position as PosProp;
use lightningcss::printer::{Printer, PrinterOptions};
use lightningcss::traits::ToCss as LightningToCss;

fn length_to_tw(prop: &impl LightningToCss, rem_scale: f32) -> Option<String> {
    let mut dest = String::new();
    let mut printer = Printer::new(&mut dest, PrinterOptions::default());
    if prop.to_css(&mut printer).is_ok() {
        if dest == "auto" {
            return Some("auto".to_string());
        }
        if dest.ends_with("rem") {
            if let Ok(val) = dest[..dest.len()-3].parse::<f32>() {
                return Some(format!("{}", val * rem_scale));
            }
        }
        if dest.ends_with("px") {
            if let Ok(val) = dest[..dest.len()-2].parse::<f32>() {
                return Some(format!("{}", (val / 16.0) * rem_scale));
            }
        }
        if dest.ends_with("%") {
            if dest == "100%" { return Some("full".to_string()); }
            if dest == "50%" { return Some("1/2".to_string()); }
            return Some(format!("[{}]", dest));
        }
        // Fallback for custom values
        return Some(format!("[{}]", dest));
    }
    None
}
use crate::tailwind::variant::TailwindVariant;
use std::collections::HashMap;
use regex::Regex;

fn resolve_vars(value: &str, map: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    let re = Regex::new(r"var\((--[^,)]+)(?:,\s*([^)]+))?\)").unwrap();
    
    for _ in 0..5 {
        let mut changed = false;
        let mut new_result = String::new();
        let mut last_end = 0;
        
        for cap in re.captures_iter(&result) {
            let full_match = cap.get(0).unwrap();
            let name = cap.get(1).unwrap().as_str();
            
            new_result.push_str(&result[last_end..full_match.start()]);
            
            if let Some(val) = map.get(name) {
                new_result.push_str(val);
                changed = true;
            } else if let Some(fallback) = cap.get(2) {
                new_result.push_str(fallback.as_str());
                changed = true;
            } else {
                new_result.push_str(full_match.as_str());
            }
            last_end = full_match.end();
        }
        
        new_result.push_str(&result[last_end..]);
        result = new_result;
        
        if !changed { break; }
    }
    result
}

pub fn map_property(
    property: &Property,
    variant: &TailwindVariant,
    rem_scale: f32,
    variable_map: &HashMap<String, String>
) -> Option<String> {
    let prefix = variant.to_prefix();
    
    // Resolve variables if any
    let mut prop_str = String::new();
    let mut printer = Printer::new(&mut prop_str, PrinterOptions::default());
    let _ = property.to_css(&mut printer, false);
    
    let resolved_str = resolve_vars(&prop_str, variable_map);
    
    // If it's a CSS variable definition, we don't map it to tailwind directly
    if prop_str.starts_with("--") {
        return None;
    }

    // If variables were resolved, we use the resolved string for arbitrary values
    if resolved_str != prop_str {
        if let Some(pos) = resolved_str.find(':') {
            let prop_name = resolved_str[..pos].trim();
            let val = resolved_str[pos + 1..].trim().trim_end_matches(';');
            
            if prop_name == "background-color" || prop_name == "background" {
                return Some(format!("{}bg-[{}]", prefix, val));
            } else if prop_name == "color" {
                return Some(format!("{}text-[{}]", prefix, val));
            } else if prop_name.starts_with("padding") {
                let side = match prop_name {
                    "padding-top" => "t",
                    "padding-bottom" => "b",
                    "padding-left" => "l",
                    "padding-right" => "r",
                    _ => "",
                };
                return Some(format!("{}p{}-[{}]", prefix, side, val));
            } else if prop_name.starts_with("margin") {
                let side = match prop_name {
                    "margin-top" => "t",
                    "margin-bottom" => "b",
                    "margin-left" => "l",
                    "margin-right" => "r",
                    _ => "",
                };
                return Some(format!("{}m{}-[{}]", prefix, side, val));
            } else {
                return Some(format!("{}[{}]", prefix, resolved_str.replace(": ", ":").replace(' ', "_")));
            }
        }
    }

    let result = match property {
        Property::Display(display) => match display {
            Display::Keyword(DisplayKeyword::None) => Some("hidden".to_string()),
            Display::Pair(DisplayPair { outside, inside, is_list_item: false }) => {
                match (outside, inside) {
                    (DisplayOutside::Block, DisplayInside::Flow) => Some("block".to_string()),
                    (DisplayOutside::Inline, DisplayInside::Flow) => Some("inline".to_string()),
                    (DisplayOutside::Inline, DisplayInside::FlowRoot) => Some("inline-block".to_string()),
                    (DisplayOutside::Block, DisplayInside::Flex(_)) => Some("flex".to_string()),
                    (DisplayOutside::Inline, DisplayInside::Flex(_)) => Some("inline-flex".to_string()),
                    (DisplayOutside::Block, DisplayInside::Grid) => Some("grid".to_string()),
                    (DisplayOutside::Inline, DisplayInside::Grid) => Some("inline-grid".to_string()),
                    _ => None,
                }
            }
            _ => None,
        },
        // Common Margin
        Property::MarginTop(v) => length_to_tw(v, rem_scale).map(|s| format!("mt-{}", s)),
        Property::MarginBottom(v) => length_to_tw(v, rem_scale).map(|s| format!("mb-{}", s)),
        Property::MarginLeft(v) => length_to_tw(v, rem_scale).map(|s| format!("ml-{}", s)),
        Property::MarginRight(v) => length_to_tw(v, rem_scale).map(|s| format!("mr-{}", s)),
        Property::Margin(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("m-[{}]", dest))
            } else {
                None
            }
        },
        
        // Common Padding
        Property::PaddingTop(v) => length_to_tw(v, rem_scale).map(|s| format!("pt-{}", s)),
        Property::PaddingBottom(v) => length_to_tw(v, rem_scale).map(|s| format!("pb-{}", s)),
        Property::PaddingLeft(v) => length_to_tw(v, rem_scale).map(|s| format!("pl-{}", s)),
        Property::PaddingRight(v) => length_to_tw(v, rem_scale).map(|s| format!("pr-{}", s)),
        Property::Padding(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("p-[{}]", dest))
            } else {
                None
            }
        },

        // Sizing
        Property::Width(v) => length_to_tw(v, rem_scale).map(|s| format!("w-{}", s)),
        Property::Height(v) => length_to_tw(v, rem_scale).map(|s| format!("h-{}", s)),
        Property::MinWidth(v) => length_to_tw(v, rem_scale).map(|s| format!("min-w-{}", s)),
        Property::MinHeight(v) => length_to_tw(v, rem_scale).map(|s| format!("min-h-{}", s)),

        // Typography
        Property::FontSize(v) => length_to_tw(v, rem_scale).map(|s| format!("text-{}", s)),
        Property::Color(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("text-[{}]", dest))
            } else {
                None
            }
        },
        Property::BackgroundColor(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("bg-[{}]", dest))
            } else {
                None
            }
        },
        Property::Background(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("bg-[{}]", dest))
            } else {
                None
            }
        },
        Property::Position(pos) => match pos {
            PosProp::Static => Some("static".to_string()),
            PosProp::Relative => Some("relative".to_string()),
            PosProp::Absolute => Some("absolute".to_string()),
            PosProp::Fixed => Some("fixed".to_string()),
            PosProp::Sticky(_) => Some("sticky".to_string()),
        },
        Property::Border(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("border-[{}]", dest))
            } else {
                None
            }
        },
        Property::BorderRadius(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("rounded-[{}]", dest))
            } else {
                None
            }
        },
        Property::BorderWidth(v) => length_to_tw(v, rem_scale).map(|s| format!("border-{}", s)),
        Property::BorderColor(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("border-[{}]", dest))
            } else {
                None
            }
        },
        Property::OutlineColor(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("outline-[{}]", dest))
            } else {
                None
            }
        },
        Property::FontWeight(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "bold" | "700" => Some("font-bold".to_string()),
                    "normal" | "400" => Some("font-normal".to_string()),
                    "medium" | "500" => Some("font-medium".to_string()),
                    "semibold" | "600" => Some("font-semibold".to_string()),
                    _ => Some(format!("font-[{}]", dest)),
                }
            } else {
                None
            }
        },
        Property::Opacity(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("opacity-[{}]", dest))
            } else {
                None
            }
        },
        Property::Custom(custom) => {
            if custom.name.as_ref() == "content" {
                let mut dest = String::new();
                let mut printer = Printer::new(&mut dest, PrinterOptions::default());
                if property.to_css(&mut printer, false).is_ok() {
                    if let Some(val) = dest.strip_prefix("content:") {
                        Some(format!("content-[{}]", val.trim()))
                    } else {
                        Some(format!("content-[{}]", dest))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        },
        Property::BorderBottom(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-b-none".to_string())
                } else {
                    Some(format!("border-b-[{}]", dest))
                }
            } else {
                None
            }
        },
        Property::BorderTop(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-t-none".to_string())
                } else {
                    Some(format!("border-t-[{}]", dest))
                }
            } else {
                None
            }
        },
        Property::BorderLeft(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-l-none".to_string())
                } else {
                    Some(format!("border-l-[{}]", dest))
                }
            } else {
                None
            }
        },
        Property::BorderRight(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-r-none".to_string())
                } else {
                    Some(format!("border-r-[{}]", dest))
                }
            } else {
                None
            }
        },
        Property::Transform(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest.starts_with("scale(") {
                    let val = dest.strip_prefix("scale(").unwrap().strip_suffix(')').unwrap();
                    Some(format!("scale-{}", val))
                } else {
                    Some(format!("[transform:{}]", dest))
                }
            } else {
                None
            }
        },
        Property::BoxSizing(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("{}", dest))
            } else {
                None
            }
        },
        Property::ZIndex(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("z-[{}]", dest))
            } else {
                None
            }
        },
        _ => {
            if prop_str.starts_with("content") {
                return Some(format!("{}content-[{}]", prefix, prop_str.split_once(':').unwrap().1.trim().trim_end_matches(';').replace(' ', "_")));
            }
            if prop_str.starts_with("box-sizing") {
                let val = prop_str.split_once(':').unwrap().1.trim().trim_end_matches(';');
                return Some(val.to_string());
            }
            if prop_str.starts_with("transform") {
                let val = prop_str.split_once(':').unwrap().1.trim().trim_end_matches(';');
                return Some(format!("{}transform-[{}]", prefix, val.replace(' ', "_")));
            }
            None
        }
    };

    result.map(|s| format!("{}{}", prefix, s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tailwind::variant::TailwindVariant;

    #[test]
    fn test_map_display() {
        use lightningcss::stylesheet::{StyleSheet, ParserOptions};
        let css = ".x { display: none; } .y { display: flex; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        
        let variant = TailwindVariant::default();
        let vars = HashMap::new();
        
        let none_prop = &rules[0].declarations.declarations[0];
        assert_eq!(map_property(none_prop, &variant, 4.0, &vars), Some("hidden".to_string()));

        let flex_prop = &rules[1].declarations.declarations[0];
        assert_eq!(map_property(flex_prop, &variant, 4.0, &vars), Some("flex".to_string()));
    }

    #[test]
    fn test_map_padding_margin() {
        use lightningcss::stylesheet::{StyleSheet, ParserOptions};
        let css = ".x { padding-top: 16px; margin-bottom: 1rem; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        
        let variant = TailwindVariant::default();
        let vars = HashMap::new();
        
        let pt = &rules[0].declarations.declarations[0];
        assert_eq!(map_property(pt, &variant, 4.0, &vars), Some("pt-4".to_string()));

        let mb = &rules[0].declarations.declarations[1];
        assert_eq!(map_property(mb, &variant, 4.0, &vars), Some("mb-4".to_string()));
    }

    #[test]
    fn test_map_colors() {
        use lightningcss::stylesheet::{StyleSheet, ParserOptions};
        let css = ".x { color: red; background-color: #f00; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        
        let variant = TailwindVariant::default();
        let vars = HashMap::new();
        
        let color = &rules[0].declarations.declarations[0];
        let out = map_property(color, &variant, 4.0, &vars).unwrap();
        assert!(out.contains("#f00") || out.contains("red"));

        let bg = &rules[0].declarations.declarations[1];
        let out = map_property(bg, &variant, 4.0, &vars).unwrap();
        assert!(out.contains("bg-") && (out.contains("#f00") || out.contains("red")));
    }

    #[test]
    fn test_map_variants() {
        use lightningcss::stylesheet::{StyleSheet, ParserOptions};
        let css = ".x { padding-top: 16px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        
        let variant = TailwindVariant::Hover;
        let vars = HashMap::new();
        let pt = &rules[0].declarations.declarations[0];
        assert_eq!(map_property(pt, &variant, 4.0, &vars), Some("hover:pt-4".to_string()));
    }

    #[test]
    fn test_rem_scale_variation() {
        use lightningcss::stylesheet::{StyleSheet, ParserOptions};
        let css = ".x { padding-top: 16px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);
        
        let variant = TailwindVariant::default();
        let vars = HashMap::new();
        let pt = &rules[0].declarations.declarations[0];
        
        assert_eq!(map_property(pt, &variant, 4.0, &vars), Some("pt-4".to_string()));
        assert_eq!(map_property(pt, &variant, 1.0, &vars), Some("pt-1".to_string()));
        assert_eq!(map_property(pt, &variant, 5.0, &vars), Some("pt-5".to_string()));
    }
}

