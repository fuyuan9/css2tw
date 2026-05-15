# css2tw

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![NPM Version](https://img.shields.io/npm/v/css2tw.svg)](https://www.npmjs.com/package/css2tw)

**css2tw** is an AI-native Rust CLI tool designed to mechanically migrate legacy CSS class usage into Tailwind CSS utility classes. It prioritizes safety, performance, and seamless integration with AI coding agents.

## 📊 Conversion Coverage

`css2tw` supports many core CSS properties, but some complex features are currently under development or require manual verification.

| Category | Status | Notes |
| :--- | :--- | :--- |
| **margin/padding** | ✅ supported | full spacing scale mapping |
| **pseudo-class** | ⚠️ partial | hover/focus/active/disabled supported |
| **pseudo-element** | ✅ supported | before/after/placeholder (v4 syntax) |
| **media query** | ⚠️ partial | breakpoint mapping (sm/md/lg/xl/2xl) only |
| **CSS variables** | ⚠️ partial | requires explicit definition or token inference |
| **complex selectors** | 🚫 unsupported | combinators (+, ~, >) are rejected for safety |
| **animations** | ⚠️ partial | standard transitions and animations only |
| **dynamic classes** | ⚠️ unsafe | detected and reported, not auto-fixed |

---

## 🚀 Features

- **🎯 AI-First Precision:** Optimized for mechanical code migration. By default, it only transforms elements with existing `class` attributes to ensure predictable, non-invasive updates to legacy codebases.
- **🛡️ Safe & Deterministic:** Prioritizes correctness. Unsupported or unsafe CSS conversions are explicitly reported rather than silently or incorrectly transformed.
- **🤖 Agent-Native:** First-class support for structured JSON (`--json`) outputs and JSON Schema (`css2tw schema`), specifically designed to be consumed by AI coding agents.
- **⚡ High Performance:** Built in Rust. Utilizes parallel file processing (`rayon`) and extremely fast parsers (`lightningcss` for CSS, `oxc` for JSX/TSX).
- **🎨 Tailwind v4 Support:** Implements support for the latest Tailwind CSS v4 syntax, including the `!` suffix for `!important` declarations, optimized arbitrary value handling, and **automatic fallback to arbitrary properties (`[prop:value]`) for unknown or vendor-specific CSS properties**, ensuring maximum conversion coverage.
- **🧩 Template Fragment Support:** Robust parsing for fragmented template files (PHP, Blade, Jinja2, etc.) while protecting template tags (e.g., `{{ ... }}`).
- **📝 Conversion Tracing:** Provides a detailed "trace" for each conversion, explaining exactly which CSS rules led to the resulting Tailwind classes.
- **🔍 Element-Aware Resolution:** Correctly resolves styles by matching HTML/JSX elements against CSS rules using full document context.
- **🚫 Tag Selector Filtering:** Automatically ignores styles from selectors without classes or IDs (e.g., `div`, `p`, `*`, `:root`) by default to prevent global base styles from polluting component-level classes. Use `--include-tag-selectors` to include them.
- **⚙️ Config Discovery:** Automatically detects and extracts custom Tailwind themes (spacing, colors, screens) from `tailwind.config.{js,ts}` files.
- **📏 Configurable Theme:** Inject custom Tailwind theme values (colors, spacing) directly into the engine via CLI or JSON config.
- **💉 Explicit CSS Injection:** To ensure deterministic behavior, CSS definitions must be explicitly provided. Automatic scanning is disabled by default to prioritize control.
- **📸 Visual Regression Testing:** Built-in utilities to initialize Playwright-based VRT to verify migration safety without breaking the UI.
- **🔌 MCP Server Support:** First-class [Model Context Protocol](https://modelcontextprotocol.io/) server for seamless integration with AI agents like Claude Desktop and Cursor.

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

### 5. Visual Regression Testing (VRT)

Generate a Playwright-based VRT setup to compare the UI before and after migration.

```bash
# Initialize VRT config and test files
css2tw vrt init --url http://localhost:3000
```

See the [VRT Usage Guide](docs/vrt-usage.md) for more details.

### 6. AI Agent Integration (MCP)

Run the dedicated MCP server to allow AI agents to control `css2tw` directly.

```bash
# Run the MCP server over stdio
css2tw-mcp
```

### 7. Tailwind Configuration Discovery

Automatically detect and extract Tailwind configuration (spacing, colors, etc.) from your project.

```bash
# Output theme configuration as JSON
css2tw detect-config . --json
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
- `--stdin`: Read source content from standard input (uses `stdin.html` or `--stdin-type`).
- `--include-patched`: Include the full converted source code in the JSON report.
- `--ndjson`: Output result as a stream of JSON objects (Newline Delimited JSON). Ideal for large-scale migrations.
- `--file-only [PATH]`: Filter detailed JSON reports to specific files (repeatable).
- `--diff`: Include the `patch` field (Unified Diff format) in the JSON report for each converted file.
- `--diagnostics`: Include detailed diagnostics for unconverted items (e.g., dynamic class patterns).

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

#### `benchmark [PATH]`

Run a benchmark on the project to measure conversion performance and accuracy. Returns aggregate statistics and failure distribution.

- `[PATH]`: Path to the directory or file to benchmark.
- `--threshold <VALUE>`: Confidence threshold for conversion. Default: `0.7`.
- `--recursive`: Recursive search for files. Default: `true`.

#### `detect-config [PATH]`

Detect and analyze Tailwind configuration (spacing, colors, screens) in the current project by parsing `tailwind.config.{js,ts}` files.

- `[PATH]`: Path to the project root. Defaults to `.`.

#### `vrt init`

Initialize a Playwright-based Visual Regression Testing setup in the current directory.

- `--url <URL>`: Target URL for capture (e.g., http://localhost:3000).

#### `config`

Print the resolved configuration that `css2tw` is currently using. Outputs in JSON format when `--json` is specified.

#### `schema`

Print the JSON Schema for reports and configuration files. Outputs in JSON format.

## 🤖 Agent Workflow Compatibility

`css2tw` is architected from the ground up to be called by AI agents (like GitHub Copilot, Cursor, or custom LLM-based tools). 

- **✅ Stable Output:** Use `css2tw schema` to get the latest report format.
- **✅ MCP Support:** Connect AI agents via [Model Context Protocol](https://modelcontextprotocol.io/) for direct tool-based interaction.
- **✅ Structured Patch:** Consume the `patch` field (Unified Diff) for safe, mechanical application of changes.
- **✅ Machine Diagnostics:** Identify dynamic class patterns (clsx, template literals) via structured `diagnostics`.
- **✅ Theme Awareness:** Agents can use `detect-config` to synchronize their internal Tailwind knowledge with the project's custom theme.
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

### Benchmarking
We use `cargo xtask` to run conversion benchmarks against popular CSS frameworks. This helps us track conversion accuracy and safety over time.

To run all benchmarks and see a summary:
```bash
cargo xtask bench-all
```

To run a specific framework version:
```bash
cargo xtask bench-bootstrap-v5   # Bootstrap 5.3 (Latest)
cargo xtask bench-bootstrap-v4   # Bootstrap 4.6
cargo xtask bench-bootstrap-v3   # Bootstrap 3.4
cargo xtask bench-bulma-v0       # Bulma 0.9
cargo xtask bench-bulma-v1       # Bulma 1.0 (Latest)
cargo xtask bench-foundation    # Foundation 6.9
cargo xtask bench-skeleton      # Skeleton 2.0
cargo xtask bench-uikit         # UIkit 3.21
cargo xtask bench-semantic      # Semantic UI 2.9
```

#### Latest Benchmark Results

| Framework | Version | Safe Conversions | Total Classes | Note |
| :--- | :--- | :--- | :--- | :--- |
| **Bootstrap** | v5.3 | **100.00%** | 1,909 | Perfect conversion with Tailwind v4 |
| **Bootstrap** | v4.6 | **100.00%** | 1,326 | Perfect conversion |
| **Bootstrap** | v3.4 | **100.00%** | 705 | Perfect conversion |
| **Bulma** | v0.9 | **100.00%** | 543 | Perfect conversion |
| **Bulma** | v1.0 | **100.00%** | 1,772 | Perfect conversion with CSS variables |
| **Foundation** | v6.9 | **100.00%** | 299 | Perfect conversion |
| **Skeleton** | v2.0 | **100.00%** | 37 | Perfect conversion |
| **UIkit** | v3.21 | **100.00%** | 648 | Perfect conversion |
| **Semantic UI** | v2.9 | **100.00%** | 67 | Tested with Fomantic-UI fork |

## Understanding "Partial" Results

With the latest version of `css2tw`, most standard CSS properties and variables are now converted with 100% confidence (**Safe**). However, you might still see "Partial" results in certain complex scenarios:

1. **Ambiguous Selectors**: When a single class name is defined across multiple CSS rules or files with conflicting or additive properties. The tool merges these, but marks them for review to ensure the cascade order is preserved correctly.
2. **Malformed or Unparseable CSS**: If a property value is syntactically invalid or uses proprietary non-standard syntax that the underlying parser cannot interpret, that specific property might be omitted while others are converted.
3. **Complex Cascade Review**: Cases where the tool successfully generates Tailwind output but detects high-risk CSS patterns that might behave differently at runtime depending on the final utility order.

In essence, a "Partial" result is no longer a sign of tool limitation, but a **proactive alert** suggesting manual verification of your CSS architecture.

### Local Execution
```bash
cargo run --bin css2tw -- <command> [args]
```

## 🛡️ Stability & AI-Agent Guarantees

As a substrate for AI-native migration, `css2tw` provides the following guarantees:

### 1. Deterministic Output
Given the same input files and configuration, the tool will always produce bit-identical JSON reports and patched source code. No hidden heuristics or non-reproducible inference are used.

### 2. JSON Schema Stability
We follow Semantic Versioning (SemVer) for our JSON report schema.
- **Major**: Breaking changes to the schema structure.
- **Minor**: New optional fields or diagnostic reasons.
- **Patch**: Bug fixes in descriptions or non-functional schema updates.

### 3. Confidence Semantics
Confidence scores (0.0 - 1.0) represent the tool's certainty in the mechanical mapping:
- **0.95 - 1.0 (Safe)**: 100% deterministic mapping to Tailwind utilities or arbitrary properties. No dynamic interference.
- **0.80 - 0.95 (Suggested)**: High confidence, but involves complex merging or potential specificity nuances. Manual review recommended.
- **Below 0.80 (Unsafe)**: Low confidence, partial mapping, or ambiguous cascade detected. Human review required.

---

## 📊 Benchmark Methodology & Transparency

We verify `css2tw` against major CSS frameworks to ensure safe and predictable migrations.

### Definition of "Safe Conversion"
A conversion is considered "Safe" only if:
1. It is a static class literal (no dynamic bindings).
2. It has a direct, deterministic mapping in the Tailwind engine.
3. It does not break the intended CSS cascade (verified via specificity analysis).
4. Any ambiguity is explicitly flagged as a diagnostic rather than guessed.

### What is Excluded (Automatic Review Required)
- **Dynamic Bindings**: `clsx()`, `:class`, `[ngClass]`, template literals with expressions.
- **Complex Selectors**: Combinators (`+`, `~`, `>`) and most pseudo-elements are handled via diagnostics.
- **Runtime Styles**: Styles generated or injected via JavaScript at runtime.

### Benchmark Results (Latest)

| Framework | Files | Safe Converted | Unsafe Detected | Manual Review | Confidence (>=0.95) |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Bootstrap 5** | 120 | 1,840 | 62 | 7 | 96.5% |
| **Bulma 1.0** | 85 | 1,650 | 45 | 12 | 97.2% |
| **Foundation** | 92 | 280 | 12 | 7 | 93.4% |

*Note: "Safe Converted" refers to static classes successfully mapped with >= 0.95 confidence.*

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and our [Code of Conduct](CODE_OF_CONDUCT.md).

## License

MIT License
