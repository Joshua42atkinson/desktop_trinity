---
description: Rust development workflow for Trinity
---

## Setup (first time only)
// turbo
1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Add WASM target: `rustup target add wasm32-unknown-unknown`
3. Install cargo-leptos: `cargo install cargo-leptos`

## Daily Development
// turbo-all

1. Check syntax: `cd /home/joshua/antigravity/day_dream && cargo check -p backend`
2. Run clippy lints: `cargo clippy -p backend -- -W clippy::all`
3. Format code: `cargo fmt`
4. Run tests: `cargo test -p backend`
5. Run the app: `cargo run -p backend`

## Build Release
// turbo
1. Build optimized: `cargo build --release -p backend`
