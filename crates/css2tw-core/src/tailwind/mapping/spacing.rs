//! Spacing mapping utilities.
//!
//! Handles the conversion of CSS length units (px, rem, %) to Tailwind's
//! spacing scale or arbitrary values.

use super::constants::DEFAULT_REM_PX;
use lightningcss::printer::{Printer, PrinterOptions};
use lightningcss::traits::ToCss as LightningToCss;

/// Converts a CSS length value (px, rem, %) to a Tailwind-compatible scale value.
pub fn length_to_tw(prop: &impl LightningToCss, rem_scale: f32) -> Option<String> {
    let mut dest = String::new();
    let mut printer = Printer::new(&mut dest, PrinterOptions::default());
    if prop.to_css(&mut printer).is_ok() {
        if dest == "auto" {
            return Some("auto".to_string());
        }
        if dest.ends_with("rem") {
            if let Ok(val) = dest[..dest.len() - 3].parse::<f32>() {
                return Some(format!("{}", val * rem_scale));
            }
        }
        if dest.ends_with("px") {
            if let Ok(val) = dest[..dest.len() - 2].parse::<f32>() {
                return Some(format!("{}", (val / DEFAULT_REM_PX) * rem_scale));
            }
        }
        if dest.ends_with("%") {
            if dest == "100%" {
                return Some("full".to_string());
            }
            if dest == "50%" {
                return Some("1/2".to_string());
            }
            return Some(format!("[{}]", super::ensure_leading_zero(&dest)));
        }
        // Fallback for custom values
        return Some(format!("[{}]", super::ensure_leading_zero(&dest)));
    }
    None
}
