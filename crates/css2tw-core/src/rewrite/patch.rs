//! Patching module.
//!
//! Handles the application of text replacements to source files,
//! ensuring that modifications are applied correctly without overlapping.

use crate::source::class_usage::Span;

use crate::report::ConfidenceReport;

/// Represents a single replacement in a source file.
#[derive(Debug, Clone)]
pub struct Replacement {
    /// The span of the original text to be replaced.
    pub span: Span,
    /// The original text content.
    pub before: String,
    /// The new text content.
    pub after: String,
    /// Confidence score and reasons for this replacement.
    pub confidence: ConfidenceReport,
    /// Human-readable reasons for this specific mapping.
    pub reasons: Vec<String>,
    /// Step-by-step trace of how this replacement was determined.
    pub trace: Vec<String>,
    /// Raw CSS properties associated with this class.
    pub raw_css: Option<String>,
    /// Actionable suggestion for the user or agent.
    pub suggestion: Option<String>,
}

/// Applies a list of replacements to a source string.
/// The replacements are sorted by their start position to ensure correct application.
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
