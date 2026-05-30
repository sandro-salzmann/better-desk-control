# better-desk-control

A desktop app and headless CLI for controlling a LINAK Bluetooth standing desk (the "Desk XXXX" series).

The app talks to the desk over BLE: read the current height, drive it up and down, and watch height updates live. BLE logic lives in a shared Rust crate so the GUI and CLI drive the desk identically.

## Tech stack

- **Frontend:** React 19 + Vite + Tailwind v4 + react-aria-components
- **Backend:** Tauri 2 (Rust)
- **Shared core:** `desk-core` Rust crate (LINAK wire protocol, BLE, `DeskController` state machine)

## Project layout

The three Rust crates form a single Cargo workspace with one shared `Cargo.lock` and `/target`.

- [crates/desk-core/](crates/desk-core/): Rust lib. LINAK wire protocol, BLE, and the `DeskController` state machine. No Tauri deps.
- [src-tauri/](src-tauri/): Tauri 2 binary. Wraps `DeskController`, exposes `desk_*` commands, emits `desk-*` window events.
- [src/](src/): React 19 frontend. UI only; subscribes to events and dispatches commands.
- [desk-cli/](desk-cli/): Headless clap CLI built on the same `DeskController`.
- [docs/](docs/): Reverse-engineered protocol notes and BLE sniffing setup.

## Getting started

Prerequisites: [Rust](https://rustup.rs/), Node.js, [Yarn](https://yarnpkg.com/), and the [Tauri 2 system dependencies](https://v2.tauri.app/start/prerequisites/).

```sh
yarn install
```

### Run the app

```sh
yarn tauri dev      # full dev app (Vite + Tauri window)
yarn tauri build    # production app bundle
```

### Run the CLI

```sh
yarn build:cli      # release build of desk-cli
```

Every command except `scan` needs a desk MAC address, passed with `-a` / `--address`. Run `scan` first to discover nearby desks and their addresses.

| Command          | Description                                            |
| ---------------- | ------------------------------------------------------ |
| `scan`           | Scan for nearby desks and print their name and address |
| `height`         | Print the desk's current height                        |
| `up [seconds]`   | Hold UP for N seconds (default 1.0)                    |
| `down [seconds]` | Hold DOWN for N seconds (default 1.0)                  |
| `stop`           | Stop / release the motor                               |

```sh
desk-cli scan
desk-cli -a DF:EA:BA:E8:8E:44 height
desk-cli -a DF:EA:BA:E8:8E:44 up 0.5
desk-cli -a DF:EA:BA:E8:8E:44 stop
```

## Development

- `yarn verify`: check everything without editing (Biome + `cargo fmt --check` + clippy)
- `yarn fix`: apply edits

Both accept `:js` / `:rust` suffixes to target one side. Run `cargo` commands (`cargo clippy --workspace`, `cargo test --workspace`, or `-p <crate>`) from the repo root.

A standalone component gallery is built alongside the app ([components.html](components.html), [src/components.tsx](src/components.tsx)); open `/components.html` in the Vite dev server. It is never linked from the real app.

## Docs

- Wire protocol / LINAK characteristics: [docs/protocol.md](docs/protocol.md)
- BLE sniffing setup: [docs/ble-sniffing.md](docs/ble-sniffing.md)

## Recommended IDE setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## License

[MIT](LICENSE)
