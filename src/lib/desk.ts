// Typed wrappers over the Tauri `desk_*` commands and `desk-*` window events.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface DeskInfo {
  name: string;
  address: string;
  rssi: number | null;
}

// Which screen the app should show right after launch, decided in Rust (see
// the `desk_boot` command). The frontend renders this and then follows events.
export type BootScreen =
  | "connected"
  | "connecting"
  | "bluetooth_off"
  | "scanning";

export interface BootState {
  screen: BootScreen;
  // remembered/connected desk name, for the connecting and connected screens
  name: string | null;
  // remembered desk address, so the connecting screen can list it as a row
  address: string | null;
  height_cm: number | null;
  moving: boolean;
  // cm tolerance for "at this preset", owned by desk-core (see useDesk)
  arrive_tolerance_cm: number;
}

export type ConnectionState = "disconnected" | "connecting" | "connected";

export type BluetoothState = "ready" | "off";

export interface HeightEvent {
  cm: number;
}
export interface ConnectionEvent {
  state: ConnectionState;
  name: string | null;
}
export interface MotionEvent {
  moving: boolean;
}
export interface BluetoothEvent {
  state: BluetoothState;
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
  // drop the link without forgetting the desk (Bluetooth went off)
  drop: () => invoke<void>("desk_drop"),
  moveStart: (direction: "up" | "down") =>
    invoke<void>("desk_move_start", { direction }),
  stop: () => invoke<void>("desk_stop"),
  moveToHeight: (cm: number) => invoke<void>("desk_move_to_height", { cm }),
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
