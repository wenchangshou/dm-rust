# Repository Guidelines

## Project Structure & Module Organization
Core Rust code is in `src/`:
- `src/device/` for controller, schedulers, and node/channel orchestration.
- `src/protocols/` for protocol adapters (with schemas in `src/protocols/schemas/`).
- `src/web/` for HTTP API handlers.
- `src/config/`, `src/db/`, and `src/utils/` for config loading, optional DB access, and shared utilities.

Integration tests live in `tests/` (Rust) and `test/` (shell API scripts). Runtime/config assets are under `config/`, `data/`, `migrations/`, and `doc/`. The Vue config tool is in `config-ui/` and its build output is `dist-config/`.

## Build, Test, and Development Commands
- `cargo build` - build debug binary.
- `cargo build --release` - optimized release build.
- `cargo run -- --config config.json --log-level info` - run server with explicit config/log level.
- `cargo test --workspace --verbose` - run Rust tests.
- `cargo fmt --all -- --check` - verify formatting (matches CI).
- `cargo clippy --workspace --all-targets -- -D warnings` - lint as errors (matches CI).
- `cd config-ui && npm install && npm run build` - build Vue config UI.
- `powershell -ExecutionPolicy Bypass -File script/make-dist.ps1` - assemble Windows distribution bundle.

## Coding Style & Naming Conventions
Rust edition is 2021. Use `rustfmt` defaults (4-space indentation), `snake_case` for modules/functions/files, and `PascalCase` for structs/enums/traits. Keep protocol modules isolated (one protocol per file, e.g., `src/protocols/novastar.rs`). Treat clippy warnings as blockers before PR.

For `config-ui/`, keep TypeScript strict-friendly and Vue components in `PascalCase` (for example, `OverviewPage.vue`).

## Testing Guidelines
Add unit tests near implementation (`#[cfg(test)]`) and integration coverage in `tests/` for API behavior. Name tests descriptively (`test_app_lifecycle_and_api`). For HTTP/manual checks, reuse scripts in `test/` (for example, `test/test_http_api.sh`). Run `cargo test` locally before opening a PR.

## Commit & Pull Request Guidelines
Current history mostly uses generic subjects like `update`; prefer explicit messages such as `web: validate batchRead payload` or `protocols: fix modbus timeout retry`.

PRs should include:
- clear scope and motivation,
- linked issue/ticket (if applicable),
- test evidence (`cargo test`, `clippy`, and any script-based checks),
- API/config impact notes (sample request/response or config diff when behavior changes).

## Security & Configuration Tips
Do not commit real device credentials, production IPs, or secrets. Use sample configs (`config.example.json`, `config/config.example.json`) and keep environment-specific overrides outside version control.
