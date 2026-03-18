# Contributing to AigisCode

## Development Setup

Prerequisites:

- Rust 1.77+
- Node.js 22+ for the website

Get started:

```bash
git clone https://github.com/Draivix/aigiscode.git
cd aigiscode
cargo build
```

Verify your setup:

```bash
cargo test
cargo run --bin aigiscode -- --version
cd website && npm ci && npm run build
```

## Workflow

1. Create a branch from `main`.
2. Make the smallest coherent change.
3. Add or update Rust tests where behavior changed.
4. Run `cargo fmt` and `cargo test`.
5. Run `npm run build` in `website/` if you touched the site.
6. Open a pull request with a clear summary and verification notes.

## Style

- Prefer typed Rust models and explicit ownership boundaries.
- Keep CLI code thin.
- Keep detectors and graph logic explainable.
- Do not reintroduce Python tooling or runtime ownership.

## Reporting Bugs

Use the GitHub bug report template and include:

- `aigiscode --version`
- `cargo --version`
- OS and repository language mix
- Reproduction steps and expected behavior

## Questions

Open a GitHub Discussion if you need design context or want to propose a larger
architecture change.
