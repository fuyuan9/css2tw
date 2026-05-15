# css2tw

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![NPM Version](https://img.shields.io/npm/v/css2tw.svg)](https://www.npmjs.com/package/css2tw)

**css2tw** is an AI-native Rust CLI tool designed to mechanically migrate legacy CSS class usage into Tailwind CSS utility classes. It prioritizes safety, performance, and seamless integration with AI coding agents.

## 📊 Conversion Coverage

`css2tw` supports many core CSS properties, but some complex features are currently under development or require manual verification.

| Category | Property Examples | Status | Notes |
| :--- | :--- | :--- | :--- |
| **Colors** | `color`, `background-color`, `border-color` | ✅ Supported | Supports both standard colors and arbitrary values (`[...]`). |
| **Spacing** | `margin`, `padding`, `gap` | ✅ Supported | Supports `rem` conversion and negative values. |
| **Sizing** | `width`, `height`, `min-width` | ✅ Supported | Supports `rem` and `px` unit conversion. |
| **Display** | `display: flex`, `grid`, `none` | ✅ Supported | Maps core layout modes to Tailwind utilities. |
| **Position** | `absolute`, `relative`, `fixed`, `sticky` | ✅ Supported | Maps standard positioning to utilities. |
| **Typography** | `font-size`, `font-weight`, `line-height` | ✅ Supported | Maps to standard scales or arbitrary values. |
| **Border** | `border-radius`, `border-width`, `border-style` | ✅ Supported | Decomposes shorthand properties for conversion. |
| **Media Queries** | `@media` (min/max, dark, print, etc.) | ✅ Supported | Maps to breakpoints, dark:, print:, and more. |
| **Pseudo Elements** | `::before`, `::after`, `::placeholder` | ✅ Supported | Supports Tailwind v4 pseudo-element syntax. |
| **Variables** | `var(--my-color)` | ✅ Supported | Resolves variable definitions across stylesheets. |
| **Animations** | `animation`, `transition` | ✅ Supported | Maps standard properties to utilities or arbitrary values. |

---

## 🚀 Features

- **🎯 AI-First Precision:** Optimized for mechanical code migration. By default, it only transforms elements with existing `class` attributes to ensure predictable, non-invasive updates to legacy codebases.
- **🛡️ Safe & Deterministic:** Prioritizes correctness. Unsupported or unsafe CSS conversions are explicitly reported rather than silently or incorrectly transformed.
- **🤖 Agent-Native:** First-class support for structured JSON (`--json`) outputs and JSON Schema (`css2tw schema`), specifically designed to be consumed by AI coding agents.
- **⚡ High Performance:** Built in Rust. Utilizes parallel file processing (`rayon`) and extremely fast parsers (`lightningcss` for CSS, `oxc` for JSX/TSX).
- **🎨 Tailwind v4 Support:** Implements support for the latest Tailwind CSS v4 syntax, including the `!` suffix for `!important` declarations and optimized arbitrary value handling.
- **🧩 Template Fragment Support:** Robust parsing for fragmented template files (PHP, Blade, Jinja2, etc.) while protecting template tags (e.g., `{{ ... }}`).
- **📝 Conversion Tracing:** Provides a detailed "trace" for each conversion, explaining exactly which CSS rules led to the resulting Tailwind classes.
- **🔍 Element-Aware Resolution:** Correctly resolves styles by matching HTML/JSX elements against CSS rules using full document context.
- **🚫 Tag Selector Filtering:** Automatically ignores styles from selectors without classes or IDs (e.g., `div`, `p`, `*`, `:root`) by default to prevent global base styles from polluting component-level classes. Use `--include-tag-selectors` to include them.
- **📏 Configurable Theme:** Inject custom Tailwind theme values (colors, spacing) directly into the engine via CLI or JSON config.
- **💉 Explicit CSS Injection:** To ensure deterministic behavior, CSS definitions must be explicitly provided. Automatic scanning is disabled by default to prioritize control.

## Installation

(Note: This is under active development. Installation commands will be updated upon release.)

```bash
cargo install css2tw-cli
```

## Usage

### 1. Scan a Repository

Analyze a repository and report convertible classes. CSS context must be explicitly provided.

```bash
# Scan using a specific CSS file
css2tw scan ./src --css-file styles.css --json
```

### 2. Convert Classes

Perform migration planning and optionally write changes.

```bash
# Preview changes (dry-run) with custom REM scale (1rem = 4 units)
css2tw convert ./src --rem-scale 4.0 --json

# Inject custom Tailwind config context from an agent
css2tw convert ./src --config-json '{"tailwind": {"customTheme": {"primary": "#ff0000"}}}' --json

# Apply changes with a confidence threshold
css2tw convert ./src --write --confidence-threshold 0.95
```

### 3. Support for Template Fragments (PHP, Blade, etc.)

Convert isolated template fragments by injecting external CSS definitions.

```bash
# Convert a Blade component using external CSS files
css2tw convert ./resources/views/components --css-file ./public/css/app.css --write

# Quick scan with inline CSS
css2tw scan ./templates --css-inline '.btn { padding: 1rem; }' --json
```

### 4. Fetch JSON Schema

Retrieve the JSON schema for reports and configurations to ensure stable integration.

```bash
css2tw schema
```

### 4. Explain Conversion

Explain how a specific CSS class would be converted.

```bash
css2tw explain .btn-primary --css ./src/styles.css --json
```

## 🛠️ CLI Reference

### Global Options

These options are available for all commands.

- `-V, --version`: Print version information.
- `-h, --help`: Print help information.
- `--json`: Output results in structured JSON format. **Minified by default for AI efficiency.**
- `--color`: Enable ANSI color codes in output. **Disabled by default.**
- `--pretty`: Pretty-print JSON output (human-readable).
- `--trace`: Include the detailed conversion trace in the output. **Disabled by default to save tokens.**
- `--reasons`: Include specific reasons for conversion results in the output. **Disabled by default.**
- `--include-patched`: Include the full converted source code in the JSON report.
- `--ndjson`: Output result as a stream of JSON objects (Newline Delimited JSON). Ideal for large-scale migrations.
- `--file-only [PATH]`: Filter detailed JSON reports to specific files (repeatable).
- `--stdin`: Read source content from standard input (uses `stdin.html` or `--stdin-type`).

### Commands

#### `scan [PATH]`

Analyze a repository and report convertible classes without writing any files. **Note: CSS context must be explicitly provided via `--css-file` or `--css-inline`.**

- `[PATH]`: The directory to scan. Defaults to the current directory (`.`).
- `--summary-only`: Return only the aggregate summary without individual class records.
- `--css-file <PATH>`: Path to an external CSS file to include in the conversion logic. Required for CSS-based conversion.
- `--css-inline <CSS>`: A string containing inline CSS definitions.
- `--include-tag-selectors`: Include styles from selectors without classes or IDs (e.g., `div`, `*`, `:root`) in the conversion. By default, these are ignored to prevent global styles from bloating individual element classes.

#### `convert [PATH]`

Perform migration planning and optionally write Tailwind utility classes back to source files. **Note: CSS context must be explicitly provided via `--css-file` or `--css-inline`.**

- `[PATH]`: The directory to scan. Defaults to the current directory (`.`).
- `--dry-run`: Preview changes without modifying files (output to console).
- `--write`: Overwrite source files with converted Tailwind classes.
- `--confidence-threshold <VALUE>`: Only apply conversions with confidence equal to or higher than this value (0.0 to 1.0). Default: `0.8`.
- `--rem-scale <VALUE>`: The scale factor for REM units. Default: `4` (1rem = 4 Tailwind units, e.g., `1rem` -> `4` -> `w-4`).
- `--custom-theme <KEY=VALUE>`: Inject custom theme values. Can be specified multiple times (e.g., `--custom-theme primary=#ff0000`).
- `--config-json <JSON>`: Inject custom configuration in JSON format.
- `--summary-only`: Return only the aggregate summary.
- `--css-file <PATH>`: Path to an external CSS file to include. Required for CSS-based conversion.
- `--css-inline <CSS>`: A string containing inline CSS definitions.
- `--include-tag-selectors`: Include styles from selectors without classes or IDs (e.g., `div`, `*`, `:root`) in the conversion. By default, these are ignored to prevent global styles from bloating individual element classes.

#### `explain <SELECTOR> --css <PATH>`

Explain how a specific CSS class selector would be converted to Tailwind.

- `<SELECTOR>`: The CSS class selector to explain (e.g., `.btn-primary`).
- `--css <PATH>`: **Required.** Path to the CSS file containing the selector definition.

#### `config`

Print the resolved configuration that `css2tw` is currently using. Outputs in JSON format when `--json` is specified.

#### `schema`

Print the JSON Schema for reports and configuration files. Outputs in JSON format.

## 🤖 Agent Workflow Compatibility

`css2tw` is architected from the ground up to be called by AI agents (like GitHub Copilot, Cursor, or custom LLM-based tools). 

- **✅ Stable Output:** Use `css2tw schema` to get the latest report format.
- **✅ Traceability:** Each conversion record includes a `trace` field with step-by-step reasoning.
- **✅ Context Injection:** Agents can inject extracted `tailwind.config.js` context directly into the conversion engine.
- **✅ Non-Interactive:** Perfect for automated pipelines and agentic loops.

Refer to [docs/agent-prompts.md](docs/agent-prompts.md) for detailed integration patterns and prompt examples.

## 📦 Distribution (NPM)

`css2tw` is available as a lightweight NPM package. It uses the **Optional Dependencies** pattern to deliver prebuilt native binaries for your specific platform, eliminating the need for a local Rust toolchain.

- `css2tw`: The main CLI wrapper.
- `css2tw-darwin-arm64`: Apple Silicon.
- `css2tw-darwin-x64`: Intel Mac.
- `css2tw-linux-x64`: Linux.
- `css2tw-win32-x64`: Windows.

---

## 🛠️ Development

We use `cargo xtask` for project automation.

### Build & Package
Build the binary and stage it for NPM:
```bash
cargo xtask dist
```

### Publishing
Publish all packages (requires proper permissions):
```bash
# Dry-run
cargo xtask publish-dry-run

# Real publish
cargo xtask publish
```

### Testing
Run the full test suite:
```bash
cargo test
```

### Local Execution
```bash
cargo run --bin css2tw -- <command> [args]
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT License
