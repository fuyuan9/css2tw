# Agent Usage Guide

This tool is designed to be highly interoperable with AI agents for automated codebase migrations.

## Core Tenets

1. **Deterministic & Minimal JSON Output:** Always use the `--json` flag. Output is minified and omits `trace` and `reasons` fields by default to optimize token consumption in AI agent workflows. Use `--trace` or `--reasons` only when deep reasoning is required.
2. **Read-Only by Default:** The tool operates in dry-run mode unless the `--write` flag is explicitly passed.
3. **Confidence Scoring:** `css2tw` calculates a confidence score (0.0 to 1.0) for every conversion. By default, agents should verify high-confidence conversions and request user confirmation for low-confidence ones.
4. **Tag Selector Filtering:** By default, styles from tag-only selectors (e.g., `div`, `*`) are ignored to keep component classes clean. Agents can override this with `--include-tag-selectors` if they need to migrate global base styles into utility classes.

## Example Workflows

### 1. Tailwind Config Discovery
Before migration, detect the existing Tailwind configuration to ensure consistent theme mapping:

```bash
# Outputs detected spacing, colors, and screens in JSON
css2tw detect-config . --json
```

### 2. Project Analysis & Benchmarking
Scan the project to understand the migration scope and baseline performance:

```bash
# Get overall conversion rate and failure reasons
css2tw benchmark ./src --json
```

### 3. Automated Migration Loop (Structured Patch)
For fully autonomous migrations, agents should consume the `patch` field for safe application of changes:

```bash
# Generate unified diffs in the JSON report
css2tw convert ./src --css-file ./src/app.css --diff --json
```

### 4. Handling Diagnostics
When `css2tw` encounters dynamic classes (e.g., `clsx`, template literals) in JSX, it issues `diagnostics`. Agents should:
1. Parse the `diagnostics` array in `unconverted` items.
2. If `manual_action_required` is true, present the code block to the user or use secondary reasoning to refactor.
3. Check `severity` (e.g., `Warning`, `Error`) to prioritize tasks.

### 5. Streaming Migration (NDJSON)
... (keep existing content)

## Parsing the Output

The JSON schema output by the CLI matches the `Report` struct. 
- **Patches**: Use the `patch` field (Unified Diff format) for applying changes via `patch` utility if not using `--write`.
- **Diagnostics**: Machine-readable metadata for dynamic class patterns that require AST refactoring rather than simple string replacement.
- **Theme Awareness**: Use the output of `detect-config` to synchronize your agent's internal Tailwind knowledge with the project's custom theme.
