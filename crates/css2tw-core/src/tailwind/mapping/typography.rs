use lightningcss::printer::{Printer, PrinterOptions};
use lightningcss::traits::ToCss as LightningToCss;

/// Maps font-weight keywords and values to Tailwind utility classes.
pub fn map_font_weight(v: &impl LightningToCss) -> Option<String> {
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
}

/// Maps font-size lengths to Tailwind standard keywords or arbitrary values.
pub fn map_font_size(v: &impl LightningToCss, _rem_scale: f32) -> Option<String> {
    let mut dest = String::new();
    let mut printer = Printer::new(&mut dest, PrinterOptions::default());
    if v.to_css(&mut printer).is_ok() {
        if dest.ends_with("rem") {
            if let Ok(val) = dest[..dest.len() - 3].parse::<f32>() {
                if let Some(keyword) = rem_to_keyword(val) {
                    return Some(keyword);
                }
            }
        }
        if dest.ends_with("px") {
            if let Ok(val) = dest[..dest.len() - 2].parse::<f32>() {
                if let Some(keyword) = rem_to_keyword(val / super::constants::DEFAULT_REM_PX) {
                    return Some(keyword);
                }
            }
        }
        // Fallback to arbitrary value with leading zero fix
        Some(format!("[{}]", super::ensure_leading_zero(&dest)))
    } else {
        None
    }
}

fn rem_to_keyword(rem: f32) -> Option<String> {
    // We use a small epsilon for float comparison
    let eps = 0.001;
    if (rem - 0.75).abs() < eps {
        Some("xs".to_string())
    } else if (rem - 0.875).abs() < eps {
        Some("sm".to_string())
    } else if (rem - 1.0).abs() < eps {
        Some("base".to_string())
    } else if (rem - 1.125).abs() < eps {
        Some("lg".to_string())
    } else if (rem - 1.25).abs() < eps {
        Some("xl".to_string())
    } else if (rem - 1.5).abs() < eps {
        Some("2xl".to_string())
    } else if (rem - 1.875).abs() < eps {
        Some("3xl".to_string())
    } else if (rem - 2.25).abs() < eps {
        Some("4xl".to_string())
    } else if (rem - 3.0).abs() < eps {
        Some("5xl".to_string())
    } else if (rem - 3.75).abs() < eps {
        Some("6xl".to_string())
    } else if (rem - 4.5).abs() < eps {
        Some("7xl".to_string())
    } else if (rem - 6.0).abs() < eps {
        Some("8xl".to_string())
    } else if (rem - 8.0).abs() < eps {
        Some("9xl".to_string())
    } else {
        None
    }
}
