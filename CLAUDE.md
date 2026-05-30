# CLAUDE.md

## What this is

A Tauri 2 desktop app that controls a LINAK Bluetooth standing desk (the "Desk XXXX" series), plus a headless CLI for the same desk. React 19 + Vite + Tailwind v4 + react-aria-components on the frontend, Rust on the backend.

## Map of the codebase

BLE logic is factored into a shared Rust crate so the GUI and CLI drive the desk identically.

- [crates/desk-core/](crates/desk-core/) — Rust lib. LINAK wire protocol, BLE, and the `DeskController` state machine. No Tauri deps.
- [src-tauri/](src-tauri/) — Tauri 2 binary. Wraps `DeskController`, exposes `desk_*` commands, emits `desk-*` window events.
- [src/](src/) — React 19 frontend. UI only; subscribes to events and dispatches commands.
- [desk-cli/](desk-cli/) — Headless clap CLI built on the same `DeskController`.
- [docs/](docs/) — Reverse-engineered protocol notes and BLE sniffing setup.

A standalone component gallery is built alongside the app ([components.html](components.html) → [src/components.tsx](src/components.tsx)). Open `/components.html` in the Vite dev server. It is never linked from the real app.

## How to run things

- `yarn tauri dev` — full dev app (Vite + Tauri window)
- `yarn tauri build` — production app bundle (uses `--locked`)
- `yarn build:cli` — release build of `desk-cli`
- `yarn verify` — check everything without editing (Biome + `cargo fmt --check` + clippy). `yarn fix` applies edits. Both take `:js` / `:rust` suffixes to target one side. Biome 2 owns code style; do not relitigate it in prose.

The three Rust crates form a single Cargo workspace ([Cargo.toml](Cargo.toml)): `desk-core`, `src-tauri`, and `desk-cli`. Run `cargo` commands from the repo root — `cargo clippy --workspace`, `cargo fmt --all`, `cargo test --workspace`, or `-p <crate>` to target one. There is one shared root `Cargo.lock` and a shared `/target`.

## Conventions

- **Rust owns control-flow decisions.** Anything that decides "what should happen now" (boot screen, reconnect vs scan, busy-state arbitration) lives in `desk-core` or [src-tauri/src/desk.rs](src-tauri/src/desk.rs). React renders state and dispatches commands; it does not branch on backend conditions to decide what the backend should do.

## Where to look first

- Wire protocol / LINAK characteristics → [docs/protocol.md](docs/protocol.md), [crates/desk-core/src/protocol.rs](crates/desk-core/src/protocol.rs)
- BLE sniffing setup → [docs/ble-sniffing.md](docs/ble-sniffing.md)
- Controller state machine (boot, hold loop, preset drive) → [crates/desk-core/src/controller/](crates/desk-core/src/controller/)
- Tauri ↔ React contract (event payloads must stay in sync) → [src-tauri/src/desk.rs](src-tauri/src/desk.rs) ↔ [src/lib/desk.ts](src/lib/desk.ts)
- App startup decision (single source of truth) → `desk_boot` in [src-tauri/src/desk.rs](src-tauri/src/desk.rs)
- UI screen/overlay routing → `appState` in [src/hooks/useDesk.ts](src/hooks/useDesk.ts) and [src/App.tsx](src/App.tsx)
- User-saved height presets → [src/lib/presets.ts](src/lib/presets.ts)
