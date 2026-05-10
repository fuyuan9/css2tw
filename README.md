# css2tw

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-v1.75+-orange.svg)](https://www.rust-lang.org/)
[![NPM Version](https://img.shields.io/npm/v/css2tw.svg)](https://www.npmjs.com/package/css2tw)

**css2tw** is an AI-native Rust CLI tool designed to mechanically migrate legacy CSS class usage into Tailwind CSS utility classes. It prioritizes safety, performance, and seamless integration with AI coding agents.

## 🚀 Features

- **🛡️ Safe & Deterministic:** Prioritizes correctness. Unsupported or unsafe CSS conversions are explicitly reported rather than silently or incorrectly transformed.
- **🤖 Agent-Native:** First-class support for structured JSON (`--json`) outputs, specifically designed to be consumed by AI coding agents.
- **⚡ High Performance:** Built in Rust. Utilizes parallel file processing (`rayon`) and extremely fast parsers (`lightningcss` for CSS, `oxc` for JSX/TSX).
- **📊 Confidence Scoring:** Applies a transparent confidence model to conversions, allowing you to filter changes based on your own risk tolerance.
- **🔍 Element-Aware Resolution:** Correctly resolves styles by matching HTML/JSX elements against CSS rules using full document context.
- **🛠️ Dry-Run by Default:** Safety first. Requires an explicit `--write` flag to mutate your source code.
- **🧩 Complex Selector Support:** Handles pseudo-classes (`:hover`, `:focus`), pseudo-elements (`::before`), and tag-based selectors (e.g., `button.btn`).
- **📏 Configurable Scale:** Support for custom REM scales via `--rem-scale` to match your Tailwind theme.

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

# Apply changes with a confidence threshold
css2tw convert ./src --write --confidence-threshold 0.95
```

### 3. Explain Conversion

Explain how a specific CSS class would be converted.

```bash
css2tw explain .btn-primary --css ./src/styles.css --json
```

## 🤖 Agent Workflow Compatibility

`css2tw` is architected from the ground up to be called by AI agents (like GitHub Copilot, Cursor, or custom LLM-based tools). 

- **✅ Stable Output:** JSON schema is strictly versioned and stable.
- **✅ Non-Interactive:** No prompts or hidden confirmations; perfect for automated pipelines.
- **✅ Structured Errors:** Errors are returned in JSON format, allowing agents to understand *why* a conversion failed.
- **✅ Reason Tracking:** Every skipped conversion includes a clear justification (e.g., "Complex Selector", "Unsupported Property").

Refer to [docs/agent-usage.md](docs/agent-usage.md) for detailed integration patterns.

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
