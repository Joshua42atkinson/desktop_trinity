# Project Dependencies

This file lists all the dependencies for the project, categorized by package.

## Backend Dependencies

- `common`: Local path dependency
- `axum`: "0.7"
- `tokio`: "1" (with "full" feature)
- `serde`: "1.0" (with "derive" feature)
- `serde_json`: "1.0"
- `tower-http`: "0.5" (with "cors" feature)
- `leptos`: "0.6" (with "ssr" feature)
- `leptos_axum`: "0.6"
- `bevy_ecs`: "0.14" (optional)
- `sqlx`: "0.7" (with "runtime-tokio-rustls", "postgres" features, optional)
- `dotenvy`: "0.15" (optional)
- `lancedb`: "0.6" (optional)
- `swarms-rs`: "0.1.5" (optional)
- `rig-core`: "0.1" (optional)
- `rocm-rs`: "0.4.2" (optional)

## Frontend Dependencies

- `common`: Local path dependency
- `leptos`: "0.6" (with "csr", "macros" features)
- `leptos_router`: "0.6" (with "csr" feature)
- `reqwest`: "0.12" (with "json" feature)
- `wasm-bindgen`: "0.2"
- `console_log`: "1.0"
- `log`: "0.4"

## Common Dependencies

- `serde`: "1.0" (with "derive" feature)
- `serde_json`: "1.0"
- `once_cell`: "1.19"
- `leptos`: "0.6" (with "serde" feature)
