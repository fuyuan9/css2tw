pub mod html;
pub mod jsx;
pub mod generic;
pub mod class_usage;

use crate::error::Css2TwError;
#[cfg(feature = "cli")]
use ignore::WalkBuilder;
#[cfg(feature = "cli")]
use rayon::prelude::*;
use std::path::{Path, PathBuf};

pub struct SourceFile {
    pub path: String,
    pub content: String,
}

pub trait ClassUsageParser {
    fn extract_classes(&self, source: &SourceFile) -> Result<Vec<class_usage::ClassUsage>, Css2TwError>;
}

pub struct Scanner;

impl Scanner {
    #[cfg(feature = "cli")]
    pub fn scan_directory<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Css2TwError> {
        let walker = WalkBuilder::new(path)
            .hidden(false) // Don't ignore hidden files by default, let config decide
            .build();

        let files: Vec<PathBuf> = walker
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
            .map(|entry| entry.into_path())
            .collect();

        Ok(files)
    }

    #[cfg(feature = "cli")]
    pub fn read_files_parallel(paths: &[PathBuf]) -> Vec<SourceFile> {
        paths
            .par_iter()
            .filter_map(|path| {
                std::fs::read_to_string(path).ok().map(|content| SourceFile {
                    path: path.to_string_lossy().to_string(),
                    content,
                })
            })
            .collect()
    }
}
