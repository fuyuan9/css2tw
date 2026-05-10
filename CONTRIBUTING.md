# Contributing to css2tw

First off, thank you for considering contributing to `css2tw`! It's people like you that make the open-source community such an amazing place to learn, inspire, and create.

## Code of Conduct

By participating in this project, you are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs

If you find a bug, please open an issue using the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md). Please include as much detail as possible to help us reproduce the issue.

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. You can use the [Feature Request template](.github/ISSUE_TEMPLATE/feature_request.md) to provide details about your idea.

### Pull Requests

1.  Fork the repository and create your branch from `main`.
2.  If you've added code that should be tested, add tests.
3.  Ensure the test suite passes.
4.  Make sure your code lints.
5.  Issue that Pull Request!

## Development Setup

### Rust Core

You will need the Rust toolchain installed.

```bash
# Build the project
cargo build

# Run tests
cargo test
```

### Automation with `xtask`

We use `cargo xtask` for various automation tasks:

```bash
# Build prebuilt binaries for npm distribution
cargo xtask dist
```

### NPM Wrapper

The `npm` directory contains the JavaScript wrapper.

```bash
cd npm
npm install
```

## Questions?

If you have any questions, feel free to open a [GitHub Issue](https://github.com/fuyuan9/css2tw/issues).
