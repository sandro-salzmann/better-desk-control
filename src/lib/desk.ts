// Typed wrappers over the Tauri `desk_*` commands and `desk-*` window events.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface DeskInfo {
  name: string;
  address: string;
  rssi: number | null;
}

export interface DeskSnapshot {
  connected: boolean;
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
  snapshot: () => invoke<DeskSnapshot>("desk_snapshot"),
  bluetoothState: () => invoke<BluetoothState>("bluetooth_state"),
  scanStart: () => invoke<void>("desk_scan_start"),
  scanStop: () => invoke<void>("desk_scan_stop"),
  connect: (address: string) => invoke<boolean>("desk_connect", { address }),
  connectSaved: () => invoke<boolean>("desk_connect_saved"),
  disconnect: () => invoke<void>("desk_disconnect"),
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
