use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The main JSON output structure of the css2tw CLI.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub version: String,
    pub command: String,
    pub mode: String,
    pub summary: Summary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<ChangeFile>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unconverted: Vec<Unconverted>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

/// A statistical summary of the conversion process.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    pub files_scanned: usize,
    pub css_files_scanned: usize,
    pub source_files_scanned: usize,
    pub classes_found: usize,
    pub classes_convertible: usize,
    pub classes_partially_convertible: usize,
    pub classes_unconvertible: usize,
    pub files_changed: usize,
    pub replacements_planned: usize,
    pub warnings: usize,
    pub errors: usize,
}

/// Represents changes made or planned for a specific source file.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChangeFile {
    pub file: String,
    pub status: String,
    pub replacements: Vec<ReplacementReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patched_content: Option<String>,
}

/// Details of a single class replacement.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReplacementReport {
    pub range: RangeReport,
    pub before: String,
    pub after: String,
    pub confidence: ConfidenceReport,
    pub source_selector: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
    /// Step-by-step trace of the conversion logic
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trace: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_css: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Detailed confidence score and reasons for a replacement.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ConfidenceReport {
    pub score: f32,
    pub reasons: Vec<ConfidenceReason>,
}

/// Specific reasons that influenced the confidence score.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(tag = "type", content = "detail")]
pub enum ConfidenceReason {
    FullMatch,
    PartialMatch(Vec<String>),
    AmbiguousSelector(String),
    VariableResolved(String),
    ThemeMapping,
    ArbitraryValue,
    LowConfidenceProperty(String),
}

/// Byte range in the original source file.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RangeReport {
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Represents a CSS class or selector that could not be converted.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Unconverted {
    pub selector: String,
    pub reason: String,
    pub details: String,
    pub confidence: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<RangeReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_css: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_serialization_skips_empty() {
        let report = Report {
            version: "1.0.0".to_string(),
            command: "scan".to_string(),
            mode: "read_only".to_string(),
            summary: Summary {
                files_scanned: 1,
                css_files_scanned: 0,
                source_files_scanned: 1,
                classes_found: 0,
                classes_convertible: 0,
                classes_partially_convertible: 0,
                classes_unconvertible: 0,
                files_changed: 0,
                replacements_planned: 0,
                warnings: 0,
                errors: 0,
            },
            changes: vec![ChangeFile {
                file: "test.html".to_string(),
                status: "modified".to_string(),
                replacements: vec![ReplacementReport {
                    range: RangeReport {
                        start_byte: 0,
                        end_byte: 10,
                    },
                    before: "a".to_string(),
                    after: "b".to_string(),
                    confidence: ConfidenceReport {
                        score: 1.0,
                        reasons: vec![ConfidenceReason::FullMatch],
                    },
                    source_selector: ".test".to_string(),
                    reasons: vec![],
                    trace: vec![],
                    raw_css: None,
                    suggestion: None,
                }],
                patched_content: None,
            }],
            unconverted: vec![],
            warnings: vec![],
            errors: vec![],
        };

        let json = serde_json::to_string(&report).unwrap();
        let json_val: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Should still have changes
        assert!(json_val.get("changes").is_some());
    }
}
