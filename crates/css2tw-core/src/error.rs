//! Error handling for the css2tw core.
//!
//! Provides a centralized error type for parsing, conversion,
//! and configuration issues.

use thiserror::Error;

/// Errors that can occur during the CSS to Tailwind conversion process.
#[derive(Error, Debug)]
pub enum Css2TwError {
    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Unsafe conversion detected: {0}")]
    UnsafeConversion(String),

    #[error("Confidence threshold not met")]
    LowConfidence,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Write failed: {0}")]
    WriteFailed(#[from] std::io::Error),
}
