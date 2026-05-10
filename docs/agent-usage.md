# Agent Usage Guide

This tool is designed to be highly interoperable with AI agents for automated codebase migrations.

## Core Tenets

1. **Deterministic JSON Output:** Always use the `--json` flag to receive structured output instead of human-readable logs.
2. **Read-Only by Default:** The tool operates in dry-run mode unless the `--write` flag is explicitly passed.
3. **Confidence Scoring:** `css2tw` calculates a confidence score (0.0 to 1.0) for every conversion. By default, agents should verify high-confidence conversions and request user confirmation for low-confidence ones.

## Example Workflows

### 1. Project Analysis
Scan the project to understand the migration scope:

```bash
css2tw scan ./src --json
```

### 2. Automated Migration Loop
For fully autonomous migrations, pass `--write` and a high `--confidence-threshold`:

```bash
css2tw convert ./src --write --confidence-threshold 0.90 --json
```

### 3. Debugging a Specific Class
If an agent needs to understand why a class was mapped a certain way (or why it failed):

```bash
css2tw explain .my-custom-btn --css ./src/styles.css --json
```

## Parsing the Output

The JSON schema output by the CLI matches the `Report` struct in `crates/css2tw-core/src/report/mod.rs`.
Agents should check `summary.errors` and `summary.warnings` to determine if the run was successful.
