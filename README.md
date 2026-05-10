# css2tw

A production-grade, AI-native Rust CLI tool that mechanically converts legacy CSS class usage into Tailwind CSS utility classes.

## Features

- **Safe & Deterministic:** Prioritizes correctness. Unsupported or unsafe CSS conversions are explicitly reported rather than silently or incorrectly transformed.
- **Machine-Readable Output:** First-class support for structured JSON (`--json`) outputs, designed specifically to be consumed by AI coding agents.
- **High Performance:** Built in Rust. Utilizes parallel file processing (`rayon`) and extremely fast parsers (`lightningcss` for CSS, `oxc` for JSX/TSX).
- **Confidence Scoring:** Applies a transparent confidence model to conversions, allowing you to only write changes above a specific threshold.
- **Dry-Run by Default:** Requires explicit `--write` flag to mutate source code. 
- **Complex Selector Support:** Handles pseudo-classes (`:hover`, `:focus`), pseudo-elements (`::before`, `::after`), and tag-based selectors (e.g., `button.btn`).
- **Element-Aware Resolution:** Correctly resolves styles by matching HTML/JSX elements against CSS rules using full document context.
- **Configurable Scale:** Support for custom REM scales via `--rem-scale` to match your Tailwind theme.

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

## Agent Workflow Compatibility

`css2tw` is built to be called by AI agents.

1. Output must be stable when using `--json`.
2. No prompts or interactive confirmations.
3. Errors are structured.
4. Exposes clear reasons for skipped conversions and confidence scores.

See `docs/agent-usage.md` for more examples.

## Distribution (NPM)

`css2tw` is distributed as a lightweight NPM package containing platform-specific prebuilt binaries. This allows users to use the tool via `npm install` without requiring a Rust environment.

This project uses the **Optional Dependencies** pattern:
- `css2tw`: The main wrapper package.
- `css2tw-darwin-arm64`: Binary for Apple Silicon.
- `css2tw-darwin-x64`: Binary for Intel Mac.
- `css2tw-linux-x64`: Binary for Linux.
- `css2tw-win32-x64`: Binary for Windows.

## Development

We use `cargo xtask` for automation tasks.

### Build and Package (Local)

Build the binary for your current platform and place it in the `npm/platforms` directory:

```bash
cargo xtask dist
```

### Publish to NPM

Publish all packages to the NPM registry:

```bash
cargo xtask publish
```

### Publish Simulation

Simulate the NPM publishing process (dry-run) for all packages:

```bash
cargo xtask publish-dry-run
```

### Test

Run unit and integration tests:

```bash
cargo test
```

### Local Run (Development)

```bash
cargo run --bin css2tw -- <command> [args]
```

## License

MIT License
