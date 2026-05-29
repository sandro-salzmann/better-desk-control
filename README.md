# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Command-line tool

For driving the desk without the GUI there's a standalone CLI in
[`desk-cli`](desk-cli/README.md).

## Desk Core

The shared BLE logic lives in [`crates/desk-core`](crates/desk-core), used by
both the app and the CLI.
