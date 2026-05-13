# Agent Usage Guide

This tool is designed to be highly interoperable with AI agents for automated codebase migrations.

## Core Tenets

1. **Deterministic & Minimal JSON Output:** Always use the `--json` flag. Output is minified and omits `trace` and `reasons` fields by default to optimize token consumption in AI agent workflows. Use `--trace` or `--reasons` only when deep reasoning is required.
2. **Read-Only by Default:** The tool operates in dry-run mode unless the `--write` flag is explicitly passed.
3. **Confidence Scoring:** `css2tw` calculates a confidence score (0.0 to 1.0) for every conversion. By default, agents should verify high-confidence conversions and request user confirmation for low-confidence ones.
4. **Tag Selector Filtering:** By default, styles from tag-only selectors (e.g., `div`, `*`) are ignored to keep component classes clean. Agents can override this with `--include-tag-selectors` if they need to migrate global base styles into utility classes.

## Example Workflows

### 1. Project Analysis
Scan the project to understand the migration scope:

```bash
# Scan using a specific CSS context
css2tw scan ./src --css-file ./src/app.css --json
```

### 2. Automated Migration Loop
For fully autonomous migrations, pass `--write` and a high `--confidence-threshold`:

```bash
# Convert using specific CSS context
css2tw convert ./src --css-file ./src/app.css --write --confidence-threshold 0.90 --json
```

### 3. Debugging a Specific Class
If an agent needs to understand why a class was mapped a certain way (or why it failed):

```bash
css2tw explain .my-custom-btn --css ./src/styles.css --json
```

### 4. Streaming Migration (NDJSON)
For large-scale migrations where a single JSON report would exceed token or memory limits:

```bash
css2tw convert ./src --css-file ./styles.css --ndjson
```
Output will be streamed line-by-line:
1. `{"type":"start", ...}`
2. `{"type":"file", "data":{...}}`
3. `{"type":"summary", "data":{...}}`

### 5. In-Memory Processing (Stdin)
If an agent has source code in memory and wants to convert it without touching the disk:

```bash
cat index.html | css2tw convert --stdin --stdin-type html --css-inline ".btn { color: red; }" --json --include-patched
```
The agent can then read the `patched_content` field from the JSON output.

## Parsing the Output

The JSON schema output by the CLI matches the `Report` struct in `crates/css2tw-core/src/report/mod.rs`.
Agents should check `summary.errors` and `summary.warnings` to determine if the run was successful.
For unconverted classes, check the `raw_css` and `suggestion` fields to automate fallback logic.
