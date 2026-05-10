//! Source file rewriting and patching logic.
//!
//! This module provides the infrastructure for planning changes to source files
//! and applying those changes (patches) safely.

pub mod diff;

pub mod patch;
pub mod planner;
