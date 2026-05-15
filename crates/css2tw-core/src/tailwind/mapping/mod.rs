//! Tailwind Mapping module.
//!
//! This module contains the core logic for mapping CSS properties to Tailwind
//! utility classes. It handles length conversion (px/rem), color mapping,
//! typography, and arbitrary value generation.

use lightningcss::printer::{Printer, PrinterOptions};

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
        Property::FlexDirection(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "row" => Some("flex-row".to_string()),
                    "row-reverse" => Some("flex-row-reverse".to_string()),
                    "column" => Some("flex-col".to_string()),
                    "column-reverse" => Some("flex-col-reverse".to_string()),
                    _ => Some(format!(
                        "flex-[direction:{}]",
                        escape_arbitrary_value(&dest)
                    )),
                }
            } else {
                None
            }
        }
        Property::FlexWrap(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "wrap" => Some("flex-wrap".to_string()),
                    "wrap-reverse" => Some("flex-wrap-reverse".to_string()),
                    "nowrap" => Some("flex-nowrap".to_string()),
                    _ => Some(format!("flex-[wrap:{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::FlexGrow(v, _) => Some(format!("grow-[{}]", v)),
        Property::FlexShrink(v, _) => Some(format!("shrink-[{}]", v)),
        Property::FlexBasis(v, _) => length_to_tw(v, rem_scale).map(|s| format!("basis-{}", s)),
        Property::JustifyContent(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "flex-start" | "start" => Some("justify-start".to_string()),
                    "flex-end" | "end" => Some("justify-end".to_string()),
                    "center" => Some("justify-center".to_string()),
                    "space-between" | "between" => Some("justify-between".to_string()),
                    "space-around" | "around" => Some("justify-around".to_string()),
                    "space-evenly" | "evenly" => Some("justify-evenly".to_string()),
                    _ => Some(format!("justify-[{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::AlignItems(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "flex-start" | "start" => Some("items-start".to_string()),
                    "flex-end" | "end" => Some("items-end".to_string()),
                    "center" => Some("items-center".to_string()),
                    "baseline" => Some("items-baseline".to_string()),
                    "stretch" => Some("items-stretch".to_string()),
                    _ => Some(format!("items-[{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::AlignSelf(v, _) => {
            let mut dest = String::new();
            let mut printer = Printer::new(&mut dest, PrinterOptions::default());
            if v.to_css(&mut printer).is_ok() {
                match dest.as_str() {
                    "auto" => Some("self-auto".to_string()),
                    "flex-start" | "start" => Some("self-start".to_string()),
                    "flex-end" | "end" => Some("self-end".to_string()),
                    "center" => Some("self-center".to_string()),
                    "baseline" => Some("self-baseline".to_string()),
                    "stretch" => Some("self-stretch".to_string()),
                    _ => Some(format!("self-[{}]", escape_arbitrary_value(&dest))),
                }
            } else {
                None
            }
        }
        Property::Gap(v) => {
            let mut row = String::new();
            let mut col = String::new();
            let mut printer_row = Printer::new(&mut row, PrinterOptions::default());
            let mut printer_col = Printer::new(&mut col, PrinterOptions::default());
            if v.row.to_css(&mut printer_row).is_ok() && v.column.to_css(&mut printer_col).is_ok() {
                if row == col {
                    length_to_tw(&v.row, rem_scale).map(|s| format!("gap-{}", s))
                } else {
                    let r = length_to_tw(&v.row, rem_scale).map(|s| format!("gap-y-{}", s));
                    let c = length_to_tw(&v.column, rem_scale).map(|s| format!("gap-x-{}", s));
                    match (r, c) {
                        (Some(rv), Some(cv)) => Some(format!("{} {}", rv, cv)),
                        (Some(rv), None) => Some(rv),
                        (None, Some(cv)) => Some(cv),
                        _ => None,
                    }
                }
            } else {
                None
            }
        }
        Property::RowGap(v) => length_to_tw(v, rem_scale).map(|s| format!("gap-y-{}", s)),
        Property::ColumnGap(v) => length_to_tw(v, rem_scale).map(|s| format!("gap-x-{}", s)),
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
            let (prop_name, prop_val) = prop_str
                .split_once(':')
                .map(|(n, v)| (n.trim(), v.trim().trim_end_matches(';')))
                .unwrap_or(("", ""));

            match prop_name {
                "display" => match prop_val {
                    "none" => Some("hidden".to_string()),
                    "block" => Some("block".to_string()),
                    "inline" => Some("inline".to_string()),
                    "inline-block" => Some("inline-block".to_string()),
                    "flex" => Some("flex".to_string()),
                    "inline-flex" => Some("inline-flex".to_string()),
                    "grid" => Some("grid".to_string()),
                    "inline-grid" => Some("inline-grid".to_string()),
                    "table" => Some("table".to_string()),
                    "inline-table" => Some("inline-table".to_string()),
                    "table-row" => Some("table-row".to_string()),
                    "table-cell" => Some("table-cell".to_string()),
                    "table-column" => Some("table-column".to_string()),
                    "table-header-group" => Some("table-header-group".to_string()),
                    "table-footer-group" => Some("table-footer-group".to_string()),
                    "table-row-group" => Some("table-row-group".to_string()),
                    "table-column-group" => Some("table-column-group".to_string()),
                    "contents" => Some("contents".to_string()),
                    "list-item" => Some("list-item".to_string()),
                    _ => Some(format!("display-[{}]", escape_arbitrary_value(prop_val))),
                },
                "flex" => match prop_val {
                    "1 1 0%" | "1 1 0" | "1" => Some("flex-1".to_string()),
                    "1 1 auto" | "auto" => Some("flex-auto".to_string()),
                    "0 1 auto" | "initial" => Some("flex-initial".to_string()),
                    "none" => Some("flex-none".to_string()),
                    _ => Some(format!("flex-[{}]", escape_arbitrary_value(prop_val))),
                },
                "float" => match prop_val {
                    "left" => Some("float-left".to_string()),
                    "right" => Some("float-right".to_string()),
                    "none" => Some("float-none".to_string()),
                    "inline-start" => Some("float-left".to_string()),
                    "inline-end" => Some("float-right".to_string()),
                    _ => Some(format!("float-[{}]", escape_arbitrary_value(prop_val))),
                },
                "clear" => match prop_val {
                    "left" => Some("clear-left".to_string()),
                    "right" => Some("clear-right".to_string()),
                    "both" => Some("clear-both".to_string()),
                    "none" => Some("clear-none".to_string()),
                    "inline-start" => Some("clear-left".to_string()),
                    "inline-end" => Some("clear-right".to_string()),
                    _ => Some(format!("clear-[{}]", escape_arbitrary_value(prop_val))),
                },
                "object-fit" => match prop_val {
                    "contain" => Some("object-contain".to_string()),
                    "cover" => Some("object-cover".to_string()),
                    "fill" => Some("object-fill".to_string()),
                    "none" => Some("object-none".to_string()),
                    "scale-down" => Some("object-scale-down".to_string()),
                    _ => Some(format!("object-[{}]", escape_arbitrary_value(prop_val))),
                },
                "text-decoration" | "text-decoration-line" => match prop_val {
                    "none" => Some("no-underline".to_string()),
                    "underline" => Some("underline".to_string()),
                    "line-through" => Some("line-through".to_string()),
                    "overline" => Some("overline".to_string()),
                    _ => Some(format!("decoration-[{}]", escape_arbitrary_value(prop_val))),
                },
                "vertical-align" => match prop_val {
                    "baseline" => Some("align-baseline".to_string()),
                    "top" => Some("align-top".to_string()),
                    "middle" => Some("align-middle".to_string()),
                    "bottom" => Some("align-bottom".to_string()),
                    "text-top" => Some("align-text-top".to_string()),
                    "text-bottom" => Some("align-text-bottom".to_string()),
                    _ => Some(format!("align-[{}]", escape_arbitrary_value(prop_val))),
                },
                "white-space" => match prop_val {
                    "normal" => Some("whitespace-normal".to_string()),
                    "nowrap" => Some("whitespace-nowrap".to_string()),
                    "pre" => Some("whitespace-pre".to_string()),
                    "pre-line" => Some("whitespace-pre-line".to_string()),
                    "pre-wrap" => Some("whitespace-pre-wrap".to_string()),
                    _ => Some(format!("whitespace-[{}]", escape_arbitrary_value(prop_val))),
                },
                "word-break" => match prop_val {
                    "break-all" => Some("break-all".to_string()),
                    "keep-all" => Some("break-keep".to_string()),
                    "break-word" => Some("break-words".to_string()),
                    _ => None,
                },
                "align-content" => match prop_val {
                    "flex-start" | "start" => Some("content-start".to_string()),
                    "flex-end" | "end" => Some("content-end".to_string()),
                    "center" => Some("content-center".to_string()),
                    "space-between" | "between" => Some("content-between".to_string()),
                    "space-around" | "around" => Some("content-around".to_string()),
                    "space-evenly" | "evenly" => Some("content-evenly".to_string()),
                    "stretch" => Some("content-stretch".to_string()),
                    "baseline" => Some("content-baseline".to_string()),
                    _ => Some(format!("content-[{}]", escape_arbitrary_value(prop_val))),
                },
                "overflow" => match prop_val {
                    "auto" => Some("overflow-auto".to_string()),
                    "hidden" => Some("overflow-hidden".to_string()),
                    "visible" => Some("overflow-visible".to_string()),
                    "scroll" => Some("overflow-scroll".to_string()),
                    _ => Some(format!("overflow-[{}]", escape_arbitrary_value(prop_val))),
                },
                "overflow-x" => match prop_val {
                    "auto" => Some("overflow-x-auto".to_string()),
                    "hidden" => Some("overflow-x-hidden".to_string()),
                    "visible" => Some("overflow-x-visible".to_string()),
                    "scroll" => Some("overflow-x-scroll".to_string()),
                    _ => Some(format!("overflow-x-[{}]", escape_arbitrary_value(prop_val))),
                },
                "overflow-y" => match prop_val {
                    "auto" => Some("overflow-y-auto".to_string()),
                    "hidden" => Some("overflow-y-hidden".to_string()),
                    "visible" => Some("overflow-y-visible".to_string()),
                    "scroll" => Some("overflow-y-scroll".to_string()),
                    _ => Some(format!("overflow-y-[{}]", escape_arbitrary_value(prop_val))),
                },
                "visibility" => match prop_val {
                    "visible" => Some("visible".to_string()),
                    "hidden" | "collapse" => Some("invisible".to_string()),
                    _ => Some(format!("visibility-[{}]", escape_arbitrary_value(prop_val))),
                },
                "user-select" => Some(format!("select-{}", prop_val)),
                "pointer-events" => Some(format!("pointer-events-{}", prop_val)),
                "cursor" => Some(format!("cursor-{}", prop_val)),
                "content" => Some(format!("content-[{}]", escape_arbitrary_value(prop_val))),
                "box-sizing" => {
                    if prop_val == "border-box" {
                        Some("box-border".to_string())
                    } else if prop_val == "content-box" {
                        Some("box-content".to_string())
                    } else {
                        None
                    }
                }
                "text-align" => match prop_val {
                    "left" | "start" => Some("text-left".to_string()),
                    "right" | "end" => Some("text-right".to_string()),
                    "center" => Some("text-center".to_string()),
                    "justify" => Some("text-justify".to_string()),
                    _ => Some(format!("text-[{}]", escape_arbitrary_value(prop_val))),
                },
                "text-transform" => match prop_val {
                    "uppercase" => Some("uppercase".to_string()),
                    "lowercase" => Some("lowercase".to_string()),
                    "capitalize" => Some("capitalize".to_string()),
                    "none" => Some("normal-case".to_string()),
                    _ => Some(format!(
                        "text-transform-[{}]",
                        escape_arbitrary_value(prop_val)
                    )),
                },
                "aspect-ratio" => match prop_val {
                    "1 / 1" | "1" => Some("aspect-square".to_string()),
                    "16 / 9" => Some("aspect-video".to_string()),
                    _ => Some(format!("aspect-[{}]", escape_arbitrary_value(prop_val))),
                },
                "top" => Some(format!("top-[{}]", escape_arbitrary_value(prop_val))),
                "bottom" => Some(format!("bottom-[{}]", escape_arbitrary_value(prop_val))),
                "left" => Some(format!("left-[{}]", escape_arbitrary_value(prop_val))),
                "right" => Some(format!("right-[{}]", escape_arbitrary_value(prop_val))),
                "z-index" => Some(format!("z-[{}]", escape_arbitrary_value(prop_val))),
                "order" => match prop_val {
                    "-1" => Some("order-first".to_string()),
                    "0" => Some("order-none".to_string()),
                    "999999" | "13" => Some("order-last".to_string()),
                    _ => Some(format!("order-[{}]", escape_arbitrary_value(prop_val))),
                },
                "line-height" => match prop_val {
                    "1" => Some("leading-none".to_string()),
                    "1.25" => Some("leading-tight".to_string()),
                    "1.375" => Some("leading-snug".to_string()),
                    "1.5" => Some("leading-normal".to_string()),
                    "1.625" => Some("leading-relaxed".to_string()),
                    "2" => Some("leading-loose".to_string()),
                    _ => Some(format!("leading-[{}]", escape_arbitrary_value(prop_val))),
                },
                "font-family" => Some(format!("font-[{}]", escape_arbitrary_value(prop_val))),
                "box-shadow" => match prop_val {
                    "none" => Some("shadow-none".to_string()),
                    _ => Some(format!("shadow-[{}]", escape_arbitrary_value(prop_val))),
                },
                "transform" => Some(format!("transform-[{}]", escape_arbitrary_value(prop_val))),
                _ => None,
            }
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
            map_property(none_prop, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("hidden".to_string())
        );

        let flex_prop = &rules[1].rule.declarations.declarations[0];
        assert_eq!(
            map_property(flex_prop, false, std::slice::from_ref(&variant), 4.0, &vars),
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
            map_property(pt, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("pt-4".to_string())
        );

        let mb = &rules[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(mb, false, std::slice::from_ref(&variant), 4.0, &vars),
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
            map_property(mr, false, std::slice::from_ref(&variant), 4.0, &vars),
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
        let out = map_property(bg, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
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
        let out = map_property(color, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        assert!(out.contains("#f00") || out.contains("red"));

        let bg = &rules[0].rule.declarations.declarations[1];
        let out = map_property(bg, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
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
            map_property(pt, false, std::slice::from_ref(&variant), 4.0, &vars),
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
            map_property(pt, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("pt-4".to_string())
        );
        assert_eq!(
            map_property(pt, false, std::slice::from_ref(&variant), 1.0, &vars),
            Some("pt-1".to_string())
        );
        assert_eq!(
            map_property(pt, false, std::slice::from_ref(&variant), 5.0, &vars),
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
            map_property(none_prop, true, std::slice::from_ref(&variant), 4.0, &vars),
            Some("hidden!".to_string())
        );

        let color_prop = &rules[0].rule.declarations.important_declarations[1];
        let out =
            map_property(color_prop, true, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
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
        let out_tr =
            map_property(transform, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        // Should NOT contain spaces
        assert!(
            !out_tr.contains(' '),
            "Transform contains spaces: {}",
            out_tr
        );
        assert!(out_tr.contains('_') || !out_tr.contains("translate(10px, 20px)"));

        let border = &rules[0].rule.declarations.declarations[1];
        let out_bd =
            map_property(border, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
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
            map_property(opacity, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("opacity-[0.5]".to_string())
        );

        // margin: .5rem -> m-[0.5rem]
        let margin = &rules[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(margin, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("m-[0.5rem]".to_string())
        );

        // padding: .25% -> p-[0.25%]
        let padding = &rules[0].rule.declarations.declarations[2];
        assert_eq!(
            map_property(padding, false, std::slice::from_ref(&variant), 4.0, &vars),
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
            map_property(
                margin_neg,
                false,
                std::slice::from_ref(&variant),
                4.0,
                &vars
            ),
            Some("m-[-0.5rem]".to_string())
        );

        let opacity_neg = &rules_neg[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(
                opacity_neg,
                false,
                std::slice::from_ref(&variant),
                4.0,
                &vars
            ),
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
            map_property(fs14, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("text-sm".to_string())
        );

        // 16px -> text-base
        let fs16 = &rules[0].rule.declarations.declarations[1];
        assert_eq!(
            map_property(fs16, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("text-base".to_string())
        );

        // 12px -> text-xs
        let fs12 = &rules[0].rule.declarations.declarations[2];
        assert_eq!(
            map_property(fs12, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("text-xs".to_string())
        );

        // 15px -> text-[15px]
        let fs15 = &rules[0].rule.declarations.declarations[3];
        assert_eq!(
            map_property(fs15, false, std::slice::from_ref(&variant), 4.0, &vars),
            Some("text-[15px]".to_string())
        );

        // 1rem -> text-base
        let fs1rem = &rules[0].rule.declarations.declarations[4];
        assert_eq!(
            map_property(fs1rem, false, std::slice::from_ref(&variant), 4.0, &vars),
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
        let out_tr = map_property(tr, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        println!("out_tr: {}", out_tr);
        assert!(out_tr.contains("transition-[opacity_0.3s_ease-in-out]"));

        let tp = &rules[0].rule.declarations.declarations[1];
        let out_tp = map_property(tp, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        println!("out_tp: {}", out_tp);
        assert_eq!(out_tp, "transition-transform".to_string());

        let td = &rules[0].rule.declarations.declarations[2];
        let out_td = map_property(td, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
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
        let out_anim =
            map_property(anim, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        println!("out_anim: {}", out_anim);
        // lightningcss might reorder: animate-[1s_infinite_spin]
        assert!(
            out_anim.contains("spin") && out_anim.contains("1s") && out_anim.contains("infinite")
        );

        let an = &rules[0].rule.declarations.declarations[1];
        let out_an = map_property(an, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        println!("out_an: {}", out_an);
        assert_eq!(out_an, "animate-pulse".to_string());

        let ad = &rules[0].rule.declarations.declarations[2];
        let out_ad = map_property(ad, false, std::slice::from_ref(&variant), 4.0, &vars).unwrap();
        println!("out_ad: {}", out_ad);
        assert_eq!(out_ad, "[animation-duration:2s]".to_string());
    }
}
