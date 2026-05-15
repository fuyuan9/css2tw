# Visual Regression Testing (VRT) Guide

Converting legacy CSS to Tailwind utility classes carries a risk of subtle visual regressions. `css2tw` provides a built-in workflow to verify migration safety using Playwright.

## Workflow Overview

The recommended VRT workflow follows these steps:
1. **Initialize**: Generate VRT configuration and test files.
2. **Baseline**: Capture screenshots of the current (legacy CSS) UI.
3. **Migrate**: Run `css2tw convert --write` to apply changes.
4. **Compare**: Capture screenshots of the new UI and compare them against the baseline.

## Getting Started

### 1. Initialize VRT
Run the following command in your project root:

```bash
css2tw vrt init --url http://localhost:3000
```

This creates:
- `playwright.vrt.config.ts`: Specialized Playwright configuration for comparison.
- `tests-vrt/migrate-vrt.spec.ts`: A test script that captures and compares full-page screenshots.

### 2. Install Dependencies
Ensure you have Playwright installed in your project:

```bash
npm install -D @playwright/test
npx playwright install chromium
```

### 3. Capture Baseline
Start your development server and run the capture command:

```bash
npx playwright test -c playwright.vrt.config.ts --update-snapshots
```

### 4. Run Migration
Perform the CSS to Tailwind conversion:

```bash
css2tw convert ./src --css-file ./styles.css --write
```

### 5. Verify Results
Run the comparison test:

```bash
npx playwright test -c playwright.vrt.config.ts
```

If the UI has changed beyond the `maxDiffPixelRatio` (default 0.01), Playwright will fail and generate a visual report showing the differences.

## Best Practices
- **Network Idle**: The generated test waits for `networkidle` to ensure all assets are loaded before taking a screenshot.
- **Clean State**: Ensure your dev server has consistent data (e.g., using mocks) to avoid false positives caused by dynamic content.
- **Component-Level VRT**: For large projects, consider modifying `tests-vrt/migrate-vrt.spec.ts` to visit specific component routes rather than just the home page.
