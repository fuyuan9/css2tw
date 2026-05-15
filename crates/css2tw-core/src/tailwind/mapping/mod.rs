//! Tailwind Mapping module.
//!
//! This module contains the core logic for mapping CSS properties to Tailwind
//! utility classes. It handles length conversion (px/rem), color mapping,
//! typography, and arbitrary value generation.

use lightningcss::printer::{Printer, PrinterOptions};
use lightningcss::properties::display::{
    Display, DisplayInside, DisplayKeyword, DisplayOutside, DisplayPair,
};
use lightningcss::properties::position::Position as PosProp;
use lightningcss::properties::Property;
use lightningcss::traits::ToCss as LightningToCss;

pub mod constants;
pub mod spacing;
pub mod typography;

use spacing::length_to_tw;
use typography::{map_font_size, map_font_weight};

use crate::tailwind::variant::TailwindVariant;
use regex::Regex;
use std::collections::HashMap;

/// Resolves CSS `var()` references in a string using a provided variable map.
/// Supports recursive resolution up to 5 levels deep.
///
/// This allows the converter to handle CSS that relies on variables for
/// colors, spacing, etc., by substituting them with their actual values
/// before mapping to Tailwind.
fn resolve_vars(value: &str, map: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    // Regex to capture the variable name and an optional fallback value.
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

        if !changed {
            break;
        }
    }
    result
}

/// Escapes a value for use in Tailwind arbitrary values [...].
///
/// Ensures that a string starting with a dot has a leading zero (e.g., .5 -> 0.5),
/// which is required for valid Tailwind arbitrary value syntax.
pub(crate) fn ensure_leading_zero(value: &str) -> String {
    let mut result = String::new();
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '.'
            && (result.is_empty()
                || result.ends_with('_')
                || result.ends_with(' ')
                || result.ends_with('-'))
        {
            if let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    result.push('0');
                }
            }
        }
        result.push(c);
    }
    result
}

/// Escapes a value for use in Tailwind arbitrary values [...]
/// Replaces spaces with underscores, removes double quotes, and ensures
/// leading dots have a leading zero (e.g., .5rem -> 0.5rem).
fn escape_arbitrary_value(value: &str) -> String {
    let fixed = ensure_leading_zero(value);
    fixed
        .chars()
        .map(|c| if c.is_whitespace() { '_' } else { c })
        .collect::<String>()
        .replace('"', "")
}

/// Formats a spacing property with proper negative value support.
/// e.g., ("ml", "-3.75") -> "-ml-3.75"
fn format_spacing(prop: &str, value: &str) -> String {
    if let Some(stripped) = value.strip_prefix('-') {
        format!("-{}-{}", prop, stripped)
    } else {
        format!("{}-{}", prop, value)
    }
}

/// Maps a single CSS property to its equivalent Tailwind utility class.
///
/// Takes into account the current variant (e.g., hover:), the REM scale factor,
/// and any applicable CSS variables. It attempts to find a standard Tailwind
/// class first, falling back to arbitrary values `[...]` if no direct match exists.
pub fn map_property(
    property: &Property,
    important: bool,
    variants: &[TailwindVariant],
    rem_scale: f32,
    variable_map: &HashMap<String, String>,
) -> Option<String> {
    let mut prefix = String::new();
    for v in variants {
        prefix.push_str(&v.to_prefix());
    }

    // Resolve variables if any
    let mut prop_str = String::new();
    let mut printer = Printer::new(&mut prop_str, PrinterOptions::default());
    let _ = property.to_css(&mut printer, important);

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
                return Some(format!("{}bg-[{}]", prefix, escape_arbitrary_value(val)));
            } else if prop_name == "color" {
                return Some(format!("{}text-[{}]", prefix, escape_arbitrary_value(val)));
            } else if prop_name.starts_with("padding") {
                let side = match prop_name {
                    "padding-top" => "t",
                    "padding-bottom" => "b",
                    "padding-left" => "l",
                    "padding-right" => "r",
                    _ => "",
                };
                return Some(format!(
                    "{}p{}-[{}]",
                    prefix,
                    side,
                    escape_arbitrary_value(val)
                ));
            } else if prop_name.starts_with("margin") {
                let side = match prop_name {
                    "margin-top" => "t",
                    "margin-bottom" => "b",
                    "margin-left" => "l",
                    "margin-right" => "r",
                    _ => "",
                };
                return Some(format!(
                    "{}m{}-[{}]",
                    prefix,
                    side,
                    escape_arbitrary_value(val)
                ));
            } else {
                let suffix = if important { "!" } else { "" };
                return Some(format!(
                    "{}[{}{}]",
                    prefix,
                    escape_arbitrary_value(resolved_str.replace(": ", ":").as_str()),
                    suffix
                ));
            }
        }
    }

    let result = match property {
        Property::Display(display) => match display {
            Display::Keyword(DisplayKeyword::None) => Some("hidden".to_string()),
            Display::Pair(DisplayPair {
                outside,
                inside,
                is_list_item: false,
            }) => match (outside, inside) {
                (DisplayOutside::Block, DisplayInside::Flow) => Some("block".to_string()),
                (DisplayOutside::Inline, DisplayInside::Flow) => Some("inline".to_string()),
                (DisplayOutside::Inline, DisplayInside::FlowRoot) => {
                    Some("inline-block".to_string())
                }
                (DisplayOutside::Block, DisplayInside::Flex(_)) => Some("flex".to_string()),
                (DisplayOutside::Inline, DisplayInside::Flex(_)) => Some("inline-flex".to_string()),
                (DisplayOutside::Block, DisplayInside::Grid) => Some("grid".to_string()),
                (DisplayOutside::Inline, DisplayInside::Grid) => Some("inline-grid".to_string()),
                _ => None,
            },
            _ => None,
        },
        // Common Margin
        Property::MarginTop(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("mt", &s)),
        Property::MarginBottom(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("mb", &s)),
        Property::MarginLeft(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("ml", &s)),
        Property::MarginRight(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("mr", &s)),
        Property::Margin(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("m-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }

        // Common Padding
        Property::PaddingTop(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("pt", &s)),
        Property::PaddingBottom(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("pb", &s)),
        Property::PaddingLeft(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("pl", &s)),
        Property::PaddingRight(v) => length_to_tw(v, rem_scale).map(|s| format_spacing("pr", &s)),
        Property::Padding(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("p-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }

        // Sizing
        Property::Width(v) => length_to_tw(v, rem_scale).map(|s| format!("w-{}", s)),
        Property::Height(v) => length_to_tw(v, rem_scale).map(|s| format!("h-{}", s)),
        Property::MinWidth(v) => length_to_tw(v, rem_scale).map(|s| format!("min-w-{}", s)),
        Property::MinHeight(v) => length_to_tw(v, rem_scale).map(|s| format!("min-h-{}", s)),

        // Typography
        Property::FontSize(v) => map_font_size(v, rem_scale).map(|s| format!("text-{}", s)),
        Property::Color(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("text-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::BackgroundColor(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("bg-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::Background(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("bg-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
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
                Some(format!("border-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::BorderRadius(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("rounded-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::BorderWidth(v) => length_to_tw(v, rem_scale).map(|s| format!("border-{}", s)),
        Property::BorderColor(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("border-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::OutlineColor(c) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if c.to_css(&mut printer).is_ok() {
                Some(format!("outline-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::FontWeight(v) => map_font_weight(v),
        Property::Opacity(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("opacity-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::Custom(custom) => {
            if custom.name.as_ref() == "content" {
                let mut dest = String::new();
                let mut printer = Printer::new(&mut dest, PrinterOptions::default());
                if property.to_css(&mut printer, false).is_ok() {
                    if let Some(val) = dest.strip_prefix("content:") {
                        Some(format!("content-[{}]", escape_arbitrary_value(val.trim())))
                    } else {
                        Some(format!("content-[{}]", escape_arbitrary_value(&dest)))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        Property::BorderBottom(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-b-none".to_string())
                } else {
                    Some(format!("border-b-[{}]", escape_arbitrary_value(&dest)))
                }
            } else {
                None
            }
        }
        Property::BorderTop(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-t-none".to_string())
                } else {
                    Some(format!("border-t-[{}]", escape_arbitrary_value(&dest)))
                }
            } else {
                None
            }
        }
        Property::BorderLeft(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-l-none".to_string())
                } else {
                    Some(format!("border-l-[{}]", escape_arbitrary_value(&dest)))
                }
            } else {
                None
            }
        }
        Property::BorderRight(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest == "none" {
                    Some("border-r-none".to_string())
                } else {
                    Some(format!("border-r-[{}]", escape_arbitrary_value(&dest)))
                }
            } else {
                None
            }
        }
        Property::Transform(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                if dest.starts_with("scale(") {
                    let val = dest
                        .strip_prefix("scale(")
                        .unwrap()
                        .strip_suffix(')')
                        .unwrap();
                    Some(format!("scale-{}", val))
                } else {
                    Some(format!("[transform:{}]", escape_arbitrary_value(&dest)))
                }
            } else {
                None
            }
        }
        Property::BackgroundImage(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("bg-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::BoxSizing(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(dest.to_string())
            } else {
                None
            }
        }
        Property::ZIndex(v) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("z-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::Transition(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("transition-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::TransitionProperty(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "all" => Some("transition-all".to_string()),
                    "none" => Some("transition-none".to_string()),
                    "opacity" => Some("transition-opacity".to_string()),
                    "transform" => Some("transition-transform".to_string()),
                    "box-shadow" => Some("transition-shadow".to_string()),
                    _ => Some(format!("transition-[{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::TransitionDuration(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("duration-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::TransitionTimingFunction(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "linear" => Some("ease-linear".to_string()),
                    "ease-in" => Some("ease-in".to_string()),
                    "ease-out" => Some("ease-out".to_string()),
                    "ease-in-out" => Some("ease-in-out".to_string()),
                    _ => Some(format!("ease-[{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::TransitionDelay(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("delay-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::Animation(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!("animate-[{}]", escape_arbitrary_value(&dest)))
            } else {
                None
            }
        }
        Property::AnimationName(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "none" => Some("animate-none".to_string()),
                    "spin" => Some("animate-spin".to_string()),
                    "ping" => Some("animate-ping".to_string()),
                    "pulse" => Some("animate-pulse".to_string()),
                    "bounce" => Some("animate-bounce".to_string()),
                    _ => Some(format!("animate-[{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::AnimationDuration(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-duration:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        Property::AnimationTimingFunction(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-timing-function:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        Property::AnimationIterationCount(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-iteration-count:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        Property::AnimationDirection(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-direction:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        Property::AnimationFillMode(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-fill-mode:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        Property::AnimationPlayState(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-play-state:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        Property::AnimationDelay(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                Some(format!(
                    "[animation-delay:{}]",
                    escape_arbitrary_value(&dest)
                ))
            } else {
                None
            }
        }
        _ => {
            if prop_str.starts_with("content") {
                let val = prop_str
                    .split_once(':')
                    .map(|(_, v)| v.trim().trim_end_matches(';'))
                    .unwrap_or("");
                return Some(format!(
                    "{}content-[{}]",
                    prefix,
                    escape_arbitrary_value(val)
                ));
            }
            if prop_str.starts_with("box-sizing") {
                let val = prop_str
                    .split_once(':')
                    .unwrap()
                    .1
                    .trim()
                    .trim_end_matches(';');
                return Some(val.to_string());
            }
            if prop_str.starts_with("transform") {
                let val = prop_str
                    .split_once(':')
                    .unwrap()
                    .1
                    .trim()
                    .trim_end_matches(';');
                return Some(format!(
                    "{}transform-[{}]",
                    prefix,
                    escape_arbitrary_value(val)
                ));
            }
            None
        }
    };

    result.map(|s| {
        let suffix = if important { "!" } else { "" };
        format!("{}{}{}", prefix, s, suffix)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tailwind::variant::TailwindVariant;

    #[test]
    fn test_map_display() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { display: none; } .y { display: flex; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let none_prop = &rules[0].rule.declarations.declarations[0];
        assert_eq!(
            map_property(none_prop, false, &[variant.clone()], 4.0, &vars),
            Some("hidden".to_string())
        );

        let flex_prop = &rules[1].rule.declarations.declarations[0];
        assert_eq!(
            map_property(flex_prop, false, &[variant.clone()], 4.0, &vars),
            Some("flex".to_string())
        );
    }

    #[test]
    fn test_map_padding_margin() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { padding-top: 16px; margin-bottom: 1rem; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let pt = &rules[0].rule.declarations.declarations[0];
        assert_eq!(
            map_property(pt, false, &[variant.clone()], 4.0, &vars),
            Some("pt-4".to_string())
        );

        let mb = &rules[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(mb, false, &[variant.clone()], 4.0, &vars),
            Some("mb-4".to_string())
        );
    }

    #[test]
    fn test_map_negative_spacing() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { margin-right: -15px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let mr = &rules[0].rule.declarations.declarations[0];
        // -15px at 16px/rem and rem-scale 4.0: (-15 / 16) * 4 = -3.75
        assert_eq!(
            map_property(mr, false, &[variant.clone()], 4.0, &vars),
            Some("-mr-3.75".to_string())
        );
    }

    #[test]
    fn test_map_arbitrary_quotes() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = r#".x { background-image: url("../img.svg"); }"#;
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let bg = &rules[0].rule.declarations.declarations[0];
        let out = map_property(bg, false, &[variant.clone()], 4.0, &vars).unwrap();
        // Should have underscores instead of spaces and NO double quotes
        assert_eq!(out, "bg-[url(../img.svg)]");
    }

    #[test]
    fn test_map_colors() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { color: red; background-color: #f00; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let color = &rules[0].rule.declarations.declarations[0];
        let out = map_property(color, false, &[variant.clone()], 4.0, &vars).unwrap();
        assert!(out.contains("#f00") || out.contains("red"));

        let bg = &rules[0].rule.declarations.declarations[1];
        let out = map_property(bg, false, &[variant.clone()], 4.0, &vars).unwrap();
        assert!(out.contains("bg-") && (out.contains("#f00") || out.contains("red")));
    }

    #[test]
    fn test_map_variants() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { padding-top: 16px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::Hover;
        let vars = HashMap::new();
        let pt = &rules[0].rule.declarations.declarations[0];
        assert_eq!(
            map_property(pt, false, &[variant.clone()], 4.0, &vars),
            Some("hover:pt-4".to_string())
        );
    }

    #[test]
    fn test_rem_scale_variation() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { padding-top: 16px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();
        let pt = &rules[0].rule.declarations.declarations[0];

        assert_eq!(
            map_property(pt, false, &[variant.clone()], 4.0, &vars),
            Some("pt-4".to_string())
        );
        assert_eq!(
            map_property(pt, false, &[variant.clone()], 1.0, &vars),
            Some("pt-1".to_string())
        );
        assert_eq!(
            map_property(pt, false, &[variant.clone()], 5.0, &vars),
            Some("pt-5".to_string())
        );
    }

    #[test]
    fn test_map_important() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { display: none !important; color: red !important; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        // lightningcss might put important ones in important_declarations
        // But here we check map_property directly
        let none_prop = &rules[0].rule.declarations.important_declarations[0];
        assert_eq!(
            map_property(none_prop, true, &[variant.clone()], 4.0, &vars),
            Some("hidden!".to_string())
        );

        let color_prop = &rules[0].rule.declarations.important_declarations[1];
        let out = map_property(color_prop, true, &[variant.clone()], 4.0, &vars).unwrap();
        assert!(out.ends_with("!"));
        assert!(out.contains("text-"));
    }

    #[test]
    fn test_arbitrary_value_spacing() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { transform: translate(10px, 20px); border: 1px solid red; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let transform = &rules[0].rule.declarations.declarations[0];
        let out_tr = map_property(transform, false, &[variant.clone()], 4.0, &vars).unwrap();
        // Should NOT contain spaces
        assert!(
            !out_tr.contains(' '),
            "Transform contains spaces: {}",
            out_tr
        );
        assert!(out_tr.contains('_') || !out_tr.contains("translate(10px, 20px)"));

        let border = &rules[0].rule.declarations.declarations[1];
        let out_bd = map_property(border, false, &[variant.clone()], 4.0, &vars).unwrap();
        assert!(!out_bd.contains(' '), "Border contains spaces: {}", out_bd);
        assert_eq!(out_bd, "border-[1px_solid_red]");
    }

    #[test]
    fn test_leading_dot_values() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { opacity: .5; margin: .5rem; padding: .25%; width: .125px; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        // opacity: .5 -> opacity-[0.5]
        let opacity = &rules[0].rule.declarations.declarations[0];
        assert_eq!(
            map_property(opacity, false, &[variant.clone()], 4.0, &vars),
            Some("opacity-[0.5]".to_string())
        );

        // margin: .5rem -> m-[0.5rem]
        let margin = &rules[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(margin, false, &[variant.clone()], 4.0, &vars),
            Some("m-[0.5rem]".to_string())
        );

        // padding: .25% -> p-[0.25%]
        let padding = &rules[0].rule.declarations.declarations[2];
        assert_eq!(
            map_property(padding, false, &[variant.clone()], 4.0, &vars),
            Some("p-[0.25%]".to_string())
        );

        // Negative values
        let css_neg = ".neg { margin: -.5rem; opacity: -.1; }";
        let stylesheet_neg = StyleSheet::parse(css_neg, ParserOptions::default()).unwrap();
        let parsed_neg = crate::css::parser::ParsedStylesheet {
            ast: stylesheet_neg,
        };
        let rules_neg = crate::css::parser::extract_style_rules(&parsed_neg);

        let margin_neg = &rules_neg[0].rule.declarations.declarations[0];
        assert_eq!(
            map_property(margin_neg, false, &[variant.clone()], 4.0, &vars),
            Some("m-[-0.5rem]".to_string())
        );

        let opacity_neg = &rules_neg[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(opacity_neg, false, &[variant.clone()], 4.0, &vars),
            Some("opacity-[-0.1]".to_string())
        );
    }

    #[test]
    fn test_font_size_mapping() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { font-size: 14px; font-size: 16px; font-size: 12px; font-size: 15px; font-size: 1rem; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        // 14px -> text-sm
        let fs14 = &rules[0].rule.declarations.declarations[0];
        assert_eq!(
            map_property(fs14, false, &[variant.clone()], 4.0, &vars),
            Some("text-sm".to_string())
        );

        // 16px -> text-base
        let fs16 = &rules[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(fs16, false, &[variant.clone()], 4.0, &vars),
            Some("text-base".to_string())
        );

        // 12px -> text-xs
        let fs12 = &rules[0].rule.declarations.declarations[2];
        assert_eq!(
            map_property(fs12, false, &[variant.clone()], 4.0, &vars),
            Some("text-xs".to_string())
        );

        // 15px -> text-[15px]
        let fs15 = &rules[0].rule.declarations.declarations[3];
        assert_eq!(
            map_property(fs15, false, &[variant.clone()], 4.0, &vars),
            Some("text-[15px]".to_string())
        );

        // 1rem -> text-base
        let fs1rem = &rules[0].rule.declarations.declarations[4];
        assert_eq!(
            map_property(fs1rem, false, &[variant.clone()], 4.0, &vars),
            Some("text-base".to_string())
        );
    }

    #[test]
    fn test_map_transition() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css = ".x { transition: opacity 0.3s ease-in-out; transition-property: transform; transition-duration: 200ms; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let tr = &rules[0].rule.declarations.declarations[0];
        let out_tr = map_property(tr, false, &[variant.clone()], 4.0, &vars).unwrap();
        println!("out_tr: {}", out_tr);
        assert!(out_tr.contains("transition-[opacity_0.3s_ease-in-out]"));

        let tp = &rules[0].rule.declarations.declarations[1];
        let out_tp = map_property(tp, false, &[variant.clone()], 4.0, &vars).unwrap();
        println!("out_tp: {}", out_tp);
        assert_eq!(out_tp, "transition-transform".to_string());

        let td = &rules[0].rule.declarations.declarations[2];
        let out_td = map_property(td, false, &[variant.clone()], 4.0, &vars).unwrap();
        println!("out_td: {}", out_td);
        assert_eq!(out_td, "duration-[0.2s]".to_string());
    }

    #[test]
    fn test_map_animation() {
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let css =
            ".x { animation: spin 1s infinite; animation-name: pulse; animation-duration: 2s; }";
        let stylesheet = StyleSheet::parse(css, ParserOptions::default()).unwrap();
        let parsed = crate::css::parser::ParsedStylesheet { ast: stylesheet };
        let rules = crate::css::parser::extract_style_rules(&parsed);

        let variant = TailwindVariant::default();
        let vars = HashMap::new();

        let anim = &rules[0].rule.declarations.declarations[0];
        let out_anim = map_property(anim, false, &[variant.clone()], 4.0, &vars).unwrap();
        println!("out_anim: {}", out_anim);
        // lightningcss might reorder: animate-[1s_infinite_spin]
        assert!(
            out_anim.contains("spin") && out_anim.contains("1s") && out_anim.contains("infinite")
        );

        let an = &rules[0].rule.declarations.declarations[1];
        let out_an = map_property(an, false, &[variant.clone()], 4.0, &vars).unwrap();
        println!("out_an: {}", out_an);
        assert_eq!(out_an, "animate-pulse".to_string());

        let ad = &rules[0].rule.declarations.declarations[2];
        let out_ad = map_property(ad, false, &[variant.clone()], 4.0, &vars).unwrap();
        println!("out_ad: {}", out_ad);
        assert_eq!(out_ad, "[animation-duration:2s]".to_string());
    }
}
