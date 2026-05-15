# AI Agent Usage Guide: CSS-to-Tailwind Infrastructure

This guide explains how AI coding agents should interact with `css2tw` to perform safe, large-scale migrations.

## 1. Machine-Readable Diagnostics

Every migration plan generated via `--json` includes a `diagnostics` array. Agents **MUST** parse this array to identify risk areas.

### Diagnostic Schema
```json
{
  "severity": "warning",
  "reason": "DynamicClass",
  "message": "...",
  "location": {
    "file": "src/App.tsx",
    "line": 15,
    "column": 24
  },
  "manual_action_required": true
}
```

### Critical Reasons
- **DynamicClass**: Variable reference or complex expression detected. **Action**: Manual refactor required.
- **RuntimeClassGeneration**: Function call detected in class attribute. **Action**: Review function implementation.
- **DynamicTemplateLiteral**: Interpolation in template string. **Action**: Convert to `clsx` or static utilities.
- **NoMappingFound**: CSS property doesn't map to Tailwind. **Action**: Add custom theme or use arbitrary values.

## 2. Benchmark for Reliability

Before and after migration, run the benchmark to ensure stability.

```bash
css2tw benchmark . --threshold 0.9 --json
```

### Metrics to Monitor
- **safe_conversion_rate**: Aim for > 80%.
- **dynamic_class_detection_rate**: Should be 100% for supported frameworks.
- **manual_review_rate**: Indicates the amount of "Human-in-the-loop" work needed.

## 3. Large-Repo Compatibility Workflow

For repositories with thousands of files, follow this phased approach:

1. **Discovery Phase**: Run `css2tw benchmark` on the whole repo to estimate complexity.
2. **Analysis Phase**: Run `css2tw scan --json` on specific modules.
3. **Safe Migration**: Run `css2tw convert --write` for files with 100% safe conversion potential.
4. **Agentic Refactor**: Use the `manual-review-helper` prompt to address flagged diagnostics module by module.

## 4. MCP Integration

When using `css2tw` via an MCP (Model Context Protocol) server:
- Use `list_files` to identify source/CSS pairs.
- Use `read_file` to capture context.
- Use `css2tw_scan` (or equivalent tool) to get the machine-readable plan.
- Always verify changes with `css2tw_benchmark` after application.

## 5. Safe Failure Policy

- **Don't Guess**: If `confidence < 0.9`, do not apply automatic changes.
- **Traceability**: Always include the `trace` and `reasons` from `css2tw` in your own logs for human auditors.
- **Explainability**: Use `css2tw explain` to justify why a specific Tailwind utility was chosen.
