# css2tw

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![NPM Version](https://img.shields.io/npm/v/css2tw.svg)](https://www.npmjs.com/package/css2tw)

**css2tw** is an AI-native Rust CLI tool designed to mechanically migrate legacy CSS class usage into Tailwind CSS utility classes. It prioritizes safety, performance, and seamless integration with AI coding agents.

## 🚀 Features

- **🛡️ Safe & Deterministic:** Prioritizes correctness. Unsupported or unsafe CSS conversions are explicitly reported rather than silently or incorrectly transformed.
- **🤖 Agent-Native:** First-class support for structured JSON (`--json`) outputs and JSON Schema (`css2tw schema`), specifically designed to be consumed by AI coding agents.
- **🕸️ WASM-Compatible:** The core engine is designed to be stateless and portable, enabling execution in browser environments or Edge functions.
- **⚡ High Performance:** Built in Rust. Utilizes parallel file processing (`rayon`) and extremely fast parsers (`lightningcss` for CSS, `oxc` for JSX/TSX).
- **📝 Conversion Tracing:** Provides a detailed "trace" for each conversion, explaining exactly which CSS rules led to the resulting Tailwind classes.
- **🔍 Element-Aware Resolution:** Correctly resolves styles by matching HTML/JSX elements against CSS rules using full document context.
- **📏 Configurable Theme:** Inject custom Tailwind theme values (colors, spacing) directly into the engine via `--config-json` or `--custom-theme`.

## Installation

(Note: This is under active development. Installation commands will be updated upon release.)

```bash
cargo install css2tw-cli
```

## Usage

### 1. Scan a Repository

Analyze a repository and report convertible classes without writing files.

```bash
css2tw scan ./src --json
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

### 3. Fetch JSON Schema

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
- `--json`: Output results in structured JSON format.
- `--no-color`: Disable ANSI color codes in output.
- `--compact`: Minify JSON output (useful for reducing token count in AI agent workflows).
- `--no-trace`: Omit the detailed conversion trace from the output.
- `--no-reasons`: Omit the specific reasons for conversion results from the output.

### Commands

#### `scan [PATH]`

Analyze a repository and report convertible classes without writing any files. This is useful for initial assessment.

- `[PATH]`: The directory to scan. Defaults to the current directory (`.`).
- `--summary-only`: Return only the aggregate summary without individual class records.

#### `convert [PATH]`

Perform migration planning and optionally write Tailwind utility classes back to source files.

- `[PATH]`: The directory to scan. Defaults to the current directory (`.`).
- `--dry-run`: Preview changes without modifying files (output to console).
- `--write`: Overwrite source files with converted Tailwind classes.
- `--confidence-threshold <VALUE>`: Only apply conversions with confidence equal to or higher than this value (0.0 to 1.0). Default: `0.8`.
- `--rem-scale <VALUE>`: The scale factor for REM units. Default: `4` (1rem = 4 Tailwind units, e.g., `1rem` -> `4` -> `w-4`).
- `--custom-theme <KEY=VALUE>`: Inject custom theme values. Can be specified multiple times (e.g., `--custom-theme primary=#ff0000`).
- `--config-json <JSON>`: Inject custom configuration in JSON format.
- `--summary-only`: Return only the aggregate summary.

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
