use crate::source::class_usage::Span;

use crate::report::ConfidenceReport;

#[derive(Debug, Clone)]
pub struct Replacement {
    pub span: Span,
    pub before: String,
    pub after: String,
    pub confidence: ConfidenceReport,
    pub reasons: Vec<String>,
    pub trace: Vec<String>,
}

pub fn apply_patches(source: &str, replacements: &[Replacement]) -> String {
    // Sort replacements by start index to avoid overlapping issues
    let mut sorted_replacements = replacements.to_vec();
    sorted_replacements.sort_by_key(|r| r.span.start);

    let mut result = String::new();
    let mut last_end = 0;

    for rep in sorted_replacements {
        if rep.span.start >= last_end {
            result.push_str(&source[last_end..rep.span.start]);
            result.push_str(&rep.after);
            last_end = rep.span.end;
        }
    }

    if last_end < source.len() {
        result.push_str(&source[last_end..]);
    }

    result
}
