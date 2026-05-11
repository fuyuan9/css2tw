# AI Agent Integration Guide for css2tw

`css2tw` is designed to be an "Agent-Native" tool. This guide helps agent developers (or those writing system prompts for agents) to integrate `css2tw` effectively.

## 1. Context Extraction (Tailwind Config)

Since `css2tw` core remains stateless to stay lightweight and WASM-compatible, it relies on the calling agent to provide project-specific context.

### Recommended Prompt for Config Extraction:
> "Read the `tailwind.config.js` or `tailwind.config.ts` file. Identify custom colors, spacing, and rem-scale. Format them as a JSON object that can be passed to a CLI tool."

### Injecting into css2tw:
Pass the extracted config using the `--config-json` flag:

```bash
css2tw convert ./src --config-json '{"tailwind": {"customTheme": {"brand-red": "#ff0000"}, "remScale": 4.0}}' --json
```

## 2. Using the JSON Schema

To ensure the agent perfectly understands the tool's output, it can first fetch the schema:

```bash
css2tw schema
```

The agent should use this schema to validate its internal parsing logic.

## 3. Explainability & User Feedback

When an agent suggests a conversion, it should use the `trace` field in the JSON report to explain *why* it made that choice.

**Example Trace Output:**
```json
{
  "before": ".btn-primary",
  "after": "bg-blue-500 text-white p-4",
  "trace": [
    "Found 3 mappings for class .btn-primary",
    "Matched background-color: #3b82f6 -> bg-blue-500",
    "Matched color: #ffffff -> text-white",
    "Matched padding: 1rem -> p-4 (using rem-scale: 4)"
  ]
}
```

## 4. Error Handling & Recovery

If `css2tw` reports an unconverted class, the agent should:
1. Check the `reason` field (e.g., "Complex Selector").
2. Attempt a manual conversion or ask the user for clarification.
3. Use `css2tw explain <selector> --css <path>` to get more details on why the engine struggled.

## 5. Fragment Conversion (PHP, Blade, etc.)

When working with partial template files that don't have direct CSS references, agents should locate the relevant CSS files and inject them.

### Strategy for Agents:
1. Identify the template file to be converted (e.g., `Button.blade.php`).
2. Search the repository for relevant CSS files (e.g., `app.css`).
3. Run `css2tw` using the `--css-file` flag to provide context.

```bash
# Agent-driven conversion of a fragment
css2tw convert ./resources/views/partials/header.blade.php --css-file ./public/css/main.css --write --json
```

Alternatively, if the agent has already read the CSS content, it can pass it directly:
```bash
css2tw convert ./partial.php --css-inline ".btn { color: blue; }" --write --json
```
