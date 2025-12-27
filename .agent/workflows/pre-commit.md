---
description: Pre-commit quality checks for Trinity codebase
---
# Pre-Commit Quality Checks

Run these checks before every commit to maintain code quality.

// turbo-all

## 1. Format Code

```bash
cargo fmt --all
```

## 2. Check for Clippy Warnings

```bash
cargo clippy --workspace -- -D warnings
```

## 3. Run Tests

```bash
cargo test --workspace
```

## 4. Check Desktop Feature

```bash
cargo check -p trinity-core --features desktop
cargo check -p trinity-desktop
```

## 5. Quick Smoke Test (optional)

```bash
cargo run -p trinity-desktop -- --version
```

## All-in-One Command

```bash
cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

## Fix Common Issues

### Unused Imports

```bash
cargo clippy --fix --lib --allow-dirty
```

### Formatting Issues

```bash
cargo fmt --all
```

### Feature Combinations

```bash
# Test both desktop and default features
cargo test -p trinity-core --features desktop
cargo test -p trinity-core --no-default-features
```
