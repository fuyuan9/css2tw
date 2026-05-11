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
