// Typed wrappers over the Tauri `desk_*` commands and `desk-*` window events.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface DeskInfo {
  name: string;
  address: string;
  rssi: number | null;
}

// Which screen the app should show, decided in Rust (`Screen` in desk-core,
// derived from connection + bluetooth). Returned by `desk_boot` and re-emitted
// on every transition via `desk-screen`.
export type Screen = "connected" | "connecting" | "bluetooth_off" | "scanning";

export interface BootState {
  screen: Screen;
  // remembered/connected desk name, for the connecting and connected screens
  name: string | null;
  // remembered desk address, so the connecting screen can list it as a row
  address: string | null;
  height_cm: number | null;
  moving: boolean;
}

export type ConnectionState = "disconnected" | "connecting" | "connected";

export type BluetoothState = "ready" | "off";

export type Direction = "up" | "down";

export interface HeightEvent {
  cm: number;
}
export interface ConnectionEvent {
  state: ConnectionState;
  name: string | null;
  // desk being connected to / connected (null on disconnect). Lets the UI mark
  // the right row in the scan list without TS having to synthesize a DeskInfo.
  address: string | null;
}
export interface MotionEvent {
  moving: boolean;
  // direction the desk is being driven; null when stopping. Decided in Rust so
  // the UI's arrow doesn't depend on a possibly-stale frontend height.
  direction: Direction | null;
}
export interface BluetoothEvent {
  state: BluetoothState;
}
export interface ScreenEvent {
  screen: Screen;
}

// --- commands --------------------------------------------------------------

export const desk = {
  // decide-and-act startup: Rust reconnects to the saved desk or starts a scan,
  // and returns the screen to show while it does (see `desk_boot`).
  boot: () => invoke<BootState>("desk_boot"),
  bluetoothState: () => invoke<BluetoothState>("bluetooth_state"),
  scanStart: () => invoke<void>("desk_scan_start"),
  scanStop: () => invoke<void>("desk_scan_stop"),
  connect: (address: string, name: string) =>
    invoke<boolean>("desk_connect", { address, name }),
  disconnect: () => invoke<void>("desk_disconnect"),
  moveStart: (direction: Direction) =>
    invoke<void>("desk_move_start", { direction }),
  // press-and-hold a preset: Rust decides the direction and stops at the target
  moveToStart: (targetCm: number) =>
    invoke<void>("desk_move_to_start", { targetCm }),
  stop: () => invoke<void>("desk_stop"),
  openBluetoothSettings: () => invoke<void>("open_bluetooth_settings"),
};

// --- events ----------------------------------------------------------------

export const onHeight = (cb: (e: HeightEvent) => void): Promise<UnlistenFn> =>
  listen<HeightEvent>("desk-height", (e) => cb(e.payload));

export const onConnection = (
  cb: (e: ConnectionEvent) => void,
): Promise<UnlistenFn> =>
  listen<ConnectionEvent>("desk-connection", (e) => cb(e.payload));

export const onMotion = (cb: (e: MotionEvent) => void): Promise<UnlistenFn> =>
  listen<MotionEvent>("desk-motion", (e) => cb(e.payload));

export const onDiscovered = (cb: (e: DeskInfo) => void): Promise<UnlistenFn> =>
  listen<DeskInfo>("desk-discovered", (e) => cb(e.payload));

export const onBluetooth = (
  cb: (e: BluetoothEvent) => void,
): Promise<UnlistenFn> =>
  listen<BluetoothEvent>("desk-bluetooth", (e) => cb(e.payload));

export const onScreen = (cb: (e: ScreenEvent) => void): Promise<UnlistenFn> =>
  listen<ScreenEvent>("desk-screen", (e) => cb(e.payload));
