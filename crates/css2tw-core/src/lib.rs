//! css2tw-core
//!
//! The core engine for converting CSS rules into Tailwind CSS utility classes.
//! This crate provides the logic for parsing CSS, resolving selectors,
//! mapping properties to Tailwind, and patching source files.

pub mod config;

pub mod converter;
pub mod css;
pub mod diagnostics;
pub mod error;
pub mod report;
pub mod rewrite;
pub mod source;
pub mod tailwind;

pub use config::Config;
pub use converter::Converter;
pub use error::Css2TwError;
pub use report::Report;
