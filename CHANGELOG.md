# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-10

### Added
- Initial release of `css2tw`.
- **Tailwind v4 Important Support**: Support for `!important` declarations using the `!` suffix (e.g. `hidden!`).
- **Improved Arbitrary Value Handling**: Automatic conversion of whitespace to underscores in arbitrary values (e.g. `[transform:translate(10px,_20px)]`).
- **Global Selector Exclusion**: Added logic to ignore `*` and `:root` selectors to prevent global styles from being added as utility classes.
- Safe & Deterministic conversion of CSS classes to Tailwind CSS utilities.
- Machine-readable JSON output for AI agent compatibility.
- High performance parallel processing using Rust and `lightningcss`.
- NPM distribution with prebuilt binaries.
- GitHub Issue and Pull Request templates.
- OSS standard documents (LICENSE, CONTRIBUTING, CODE_OF_CONDUCT, SECURITY).
