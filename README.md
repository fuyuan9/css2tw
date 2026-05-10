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
