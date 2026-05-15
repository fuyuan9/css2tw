# Agent Usage Guide

This tool is designed to be highly interoperable with AI agents for automated codebase migrations.

## Core Tenets

1. **Deterministic & Minimal JSON Output:** Always use the `--json` flag. Output is minified and omits `trace` and `reasons` fields by default to optimize token consumption in AI agent workflows. Use `--trace` or `--reasons` only when deep reasoning is required.
2. **Read-Only by Default:** The tool operates in dry-run mode unless the `--write` flag is explicitly passed.
3. **Confidence Scoring:** `css2tw` calculates a confidence score (0.0 to 1.0) for every conversion. By default, agents should verify high-confidence conversions and request user confirmation for low-confidence ones.
4. **Tag Selector Filtering:** By default, styles from tag-only selectors (e.g., `div`, `*`) are ignored to keep component classes clean. Agents can override this with `--include-tag-selectors` if they need to migrate global base styles into utility classes.
5. **Safety via VRT:** Agents are encouraged to initialize [Visual Regression Testing](vrt-usage.md) before performing destructive write operations to ensure UI consistency.

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

## Safe Migration Workflow

To ensure a 100% safe migration, follow this multi-stage process:

1.  **Scan & Inventory**: Run `css2tw scan ./src --json` to identify all class usages and potential conversion targets.
2.  **Confidence Filtering**: Process only items with `confidence.score >= 0.95`. These are "safe auto-fixes".
3.  **Diagnostic Review**: Review `unconverted` items with `FailureReason::DynamicClass` or `DynamicTemplateLiteral`. These require manual AST refactoring.
4.  **Dry-Run & Diff**: Run `css2tw convert ./src --diff --json` to generate unified diffs without modifying files.
5.  **Visual Regression (VRT)**: Apply changes to a staging branch and run Playwright-based VRT to confirm no visual regressions.
6.  **PR Generation**: Group changes by component or directory and generate atomic PRs with the conversion report attached.

## Large Repository Strategy

For repositories with thousands of files, avoid processing everything at once:

-   **NDJSON Streaming**: Use `--format ndjson` to process files one by one. This prevents memory exhaustion and allows real-time progress tracking.
-   **Chunked Migration**: Migrate by directory or feature module. Start with low-impact UI components.
-   **Confidence Thresholding**: Set a strict threshold (e.g., `--min-confidence 0.98`) for initial automated batches.
-   **CI Integration**: Use `css2tw benchmark` in CI to prevent the introduction of new unconvertible CSS patterns.

## Example Prompts for AI Agents

### Cursor / GitHub Copilot (Migration Prompt)
> "Use `css2tw` to scan the current directory and identify all CSS classes that can be safely converted to Tailwind with a confidence score > 0.95. For each safe conversion, apply the change. If you encounter dynamic classes (`clsx`, template literals), list them in a summary for my review."

### Claude Code (Safe Review Prompt)
> "Run `css2tw benchmark ./src --json`. Based on the report, identify the top 3 most common failure reasons. Then, examine the code for the first failure reason and suggest a safe manual refactoring to Tailwind utilities."

### PR Generation Prompt
> "I have finished converting `Header.tsx` to Tailwind using `css2tw`. Here is the conversion report. Please generate a PR description that highlights the number of safe conversions, the unresolved dynamic patterns, and the visual verification steps taken."

## Schema Stability Policy

`css2tw` guarantees that the JSON report schema (defined in `REPORT_SCHEMA.md`) follows Semantic Versioning. 
- **Deterministic Output**: For a given input, the tool will always produce the same JSON output.
- **Backward Compatibility**: New fields may be added, but existing fields will not be renamed or removed without a major version bump.
- **Confidence Semantics**: The meaning of confidence scores is documented and will remain stable across minor versions.

## Streaming Migration (NDJSON)

For large-scale migrations, use the `--ndjson` flag to stream results as newline-delimited JSON objects. This allows external tools to process results incrementally without waiting for the entire project scan to complete.

## Parsing the Output

The JSON schema output by the CLI matches the `Report` struct. 
- **Patches**: Use the `patch` field (Unified Diff format) for applying changes via `patch` utility if not using `--write`.
- **Diagnostics**: Machine-readable metadata for dynamic class patterns that require AST refactoring rather than simple string replacement.
- **Theme Awareness**: Use the output of `detect-config` to synchronize your agent's internal Tailwind knowledge with the project's custom theme.

## Seamless Tooling via MCP

For agents that support the [Model Context Protocol](https://modelcontextprotocol.io/) (e.g., Claude Desktop, Cursor), you can connect `css2tw-mcp` as a direct tool provider. This allows the agent to call `scan_project` and `detect_config` without manual shell command generation.

**Configuration Example (Claude Desktop):**
```json
{
  "mcpServers": {
    "css2tw": {
      "command": "path/to/css2tw-mcp"
    }
  }
}
```
