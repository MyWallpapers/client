# Repository Guidelines

## Project Structure & Module Organization
This repository is a Tauri v2 desktop client for MyWallpaper. Core application code lives in `src-tauri/`, with Rust sources under `src-tauri/src/` such as `main.rs`, `lib.rs`, `commands.rs`, `window_layer.rs`, and `system_monitor.rs`. Tauri configuration and capabilities live in `src-tauri/tauri.conf.json` and `src-tauri/capabilities/`. App icons are stored in `src-tauri/icons/`. The vendored `crates/wry/` fork supports Windows-specific webview behavior and should only be changed when the desktop embedding layer requires it.

## Build, Test, and Development Commands
Use Node.js to drive Tauri commands and Cargo for Rust checks:

```bash
npm install
npm run tauri:dev
npm run tauri:build
npm run tauri:build:debug
npm run typegen
cd src-tauri && cargo check
cd src-tauri && cargo test
```

`tauri:dev` launches the desktop app against the remote frontend. `tauri:build` creates a release build, while `tauri:build:debug` keeps debug symbols for local troubleshooting. `typegen` regenerates `generated/types.ts` from shared Rust types.

## Coding Style & Naming Conventions
Rust uses `rustfmt`; the vendored formatter config in `crates/wry/rustfmt.toml` sets 2-space indentation and a 100-character line width. Follow standard Rust naming: `snake_case` for functions/modules, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Keep Tauri command names explicit, for example `get_system_info` or `download_and_install_update`. Prefer focused modules over large mixed-purpose files.

## Testing Guidelines
Run `cd src-tauri && cargo test` before opening a PR. There is no broad app-level test suite yet; existing automated tests are mainly in `crates/wry/`. For app changes, at minimum run `cargo check`, verify the Tauri app starts with `npm run tauri:dev`, and document any Windows-only manual validation. Add unit tests next to Rust modules using `#[cfg(test)]` and descriptive test names such as `reload_window_emits_event`.

## Commit & Pull Request Guidelines
Commits are enforced by `commitlint` through `lefthook`. Use Conventional Commits with allowed types including `feat`, `fix`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, and `release`, for example: `fix: correct windows crate imports`. Pull requests should include a concise summary, linked issue or context, affected platforms, and screenshots or screen recordings for tray, window-layer, or desktop-visibility changes.

## Security & Configuration Tips
Do not commit secrets. Updater signing keys belong in GitHub Actions secrets, not in the repository. Treat changes to `src-tauri/tauri.conf.json`, auto-update settings, deep-link handling, and Windows-specific APIs as security-sensitive and call them out explicitly in review.
