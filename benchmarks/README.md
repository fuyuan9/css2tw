# css2tw Benchmarks

This directory contains benchmark results and methodology for verifying the safety and accuracy of `css2tw`.

## Definition of "Safe Conversion"

A "Safe Conversion" in `css2tw` is defined by the following criteria:

1.  **Deterministic Mapping**: The mapping from CSS property to Tailwind utility is based on stable schemas and not on heuristics or guessing.
2.  **No Dynamic Interference**: The conversion target is a static string literal. Any dynamic patterns (template literals with expressions, function calls, etc.) are explicitly marked as "unsafe" and excluded from automatic conversion.
3.  **High Confidence (>= 0.95)**: Only conversions with a confidence score of 0.95 or higher are considered "safe".
4.  **No Specificity Breakage**: The conversion does not change the intended CSS cascade or specificity order in a way that breaks visual appearance.

## Benchmark Methodology

Benchmarks are run against major CSS frameworks (Bootstrap, Bulma, Foundation) and representative real-world projects.

For each benchmark, we measure:
-   **Total Files**: Number of files scanned.
-   **Safe Converted**: Number of class conversions that met the "safe" criteria.
-   **Unsafe Detected**: Number of dynamic patterns detected and safely skipped.
-   **Manual Review Required**: Number of items that could not be automatically converted with high confidence.
-   **Confidence Distribution**: Breakdown of conversion confidence across the codebase.

## How to Run Benchmarks

Run the following command to generate a benchmark report for a specific directory:

```bash
css2tw benchmark --dir ./fixtures/bootstrap --output ./benchmarks/bootstrap.json
```

## Known Limitations

-   **Runtime styles**: Styles injected via JavaScript at runtime cannot be detected or converted.
-   **Complex Selector Hacks**: Selectors using advanced CSS hacks or non-standard syntax may be skipped.
-   **Context-Dependent Specificity**: In cases where CSS specificity depends on external factors (e.g. dynamic parent classes), the tool will flag the item for manual review.
