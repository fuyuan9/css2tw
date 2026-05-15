//! Tailwind CSS mapping and variant logic.
//!
//! This module contains the logic for mapping CSS properties to Tailwind utility classes
//! and handling various Tailwind variants (hover, focus, etc.).

pub mod mapping;
pub use mapping::map_property;

pub mod detector;
pub mod variant;
