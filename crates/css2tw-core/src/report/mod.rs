pub mod schema;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub version: String,
    pub command: String,
    pub mode: String,
    pub summary: Summary,
    pub changes: Vec<ChangeFile>,
    pub unconverted: Vec<Unconverted>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeFile {
    pub file: String,
    pub status: String,
    pub replacements: Vec<ReplacementReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplacementReport {
    pub range: RangeReport,
    pub before: String,
    pub after: String,
    pub confidence: f32,
    pub source_selector: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeReport {
    pub start_byte: usize,
    pub end_byte: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unconverted {
    pub selector: String,
    pub reason: String,
    pub details: String,
    pub confidence: f32,
}
