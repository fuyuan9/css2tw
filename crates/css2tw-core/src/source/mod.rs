//! Source file analysis and class extraction.
//!
//! This module provides tools to scan directories for source files,
//! read them, and parse them to find where CSS classes are used.

pub mod class_usage;
pub mod fragment_parser;
pub mod generic;
pub mod html;
pub mod jsx;

use crate::error::Css2TwError;
#[cfg(feature = "cli")]
use ignore::WalkBuilder;
#[cfg(feature = "cli")]
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Represents a single source file (e.g., .html, .jsx) to be processed.
pub struct SourceFile {
    /// Path to the file, relative or absolute.
    pub path: String,
    /// Full text content of the file.
    pub content: String,
}

impl SourceFile {
    /// Converts a byte offset into 1-indexed (line, column).
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, c) in self.content.char_indices() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

/// Trait for different types of parsers that can extract CSS class usages from a source file.
pub trait ClassUsageParser {
    /// Extracts all class names and their spans from the given source file.
    fn extract_classes(
        &self,
        source: &SourceFile,
    ) -> Result<Vec<class_usage::ClassUsage>, Css2TwError>;
}

/// Utility for scanning directories and reading files.
pub struct Scanner;

impl Scanner {
    /// Scans a directory for all files recursively.
    #[cfg(feature = "cli")]
    pub fn scan_directory<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Css2TwError> {
        let walker = WalkBuilder::new(path)
            .hidden(false) // Don't ignore hidden files by default, let config decide
            .build();

        let files: Vec<PathBuf> = walker
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
            .map(|entry| entry.into_path())
            .collect();

        Ok(files)
    }

    /// Reads multiple files from disk in parallel and returns them as SourceFile objects.
    #[cfg(feature = "cli")]
    pub fn read_files_parallel(paths: &[PathBuf]) -> Vec<SourceFile> {
        paths
            .par_iter()
            .filter_map(|path| {
                std::fs::read_to_string(path)
                    .ok()
                    .map(|content| SourceFile {
                        path: path.to_string_lossy().to_string(),
                        content,
                    })
            })
            .collect()
    }
}
