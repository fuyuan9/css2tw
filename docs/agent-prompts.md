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
 
## 3. Migration Strategy (Tag Selectors)
 
By default, `css2tw` only converts styles from selectors that include a **class** or **ID**. This prevents global styles (like `div { margin: 0 }` or `* { box-sizing: border-box }`) from being added to every element as Tailwind utility classes.
 
### When to include Tag Selectors:
If the project relies heavily on tag-level styling and the agent needs to move all styles into utility classes:
- Pass the `--include-tag-selectors` flag.
 
### Agent Advice:
"Prefer the default (filtering) for large legacy projects to keep the HTML clean. Only enable tag selector conversion if explicitly asked to eliminate global CSS files entirely."

## 4. Explainability & User Feedback

When an agent suggests a conversion, it should use the `trace` field in the JSON report to explain *why* it made that choice. **Note: The `--trace` flag must be explicitly passed to include this field.**

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

## 5. Error Handling & Recovery

If `css2tw` reports an unconverted class, the agent should:
1. Check the `reason` field (e.g., "Complex Selector").
2. Attempt a manual conversion or ask the user for clarification.
3. Use `css2tw explain <selector> --css <path>` to get more details on why the engine struggled.

## 6. CSS Context Injection (Mandatory)

To ensure deterministic results, `css2tw` does not automatically scan for CSS files in the target directory. Agents **must** identify and provide relevant CSS context.

### Recommended Strategy for Agents:
1.  **Locate Style Definitions**: Search the repository for `.css`, `.scss`, or `.less` files that define the classes used in the target source files.
2.  **Inject via CLI**: Pass the found file paths using `--css-file`.

```bash
# Agent-driven conversion with explicit CSS context
css2tw convert ./src/components --css-file ./src/styles/main.css --write --json
```

3.  **Use Inline CSS**: If the agent has already extracted specific rules, it can pass them directly using `--css-inline`.

```bash
css2tw scan ./src --css-inline ".btn-red { color: red; }" --json
```
