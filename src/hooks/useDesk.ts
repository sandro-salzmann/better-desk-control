// The app's desk/connection state. Rust decides *what to do* on launch and on
// a Bluetooth recovery (reconnect to the saved desk vs. scan) in the
// `desk_boot` command and *what screen to show* in `desk-screen` events; this
// hook only stores those decisions, subscribes to the per-fact events, and
// exposes actions.

import type { UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  type BootState,
  type ConnectionState,
  type DeskInfo,
  type Direction,
  desk,
  onBluetooth,
  onConnection,
  onDiscovered,
  onHeight,
  onMotion,
  onScreen,
  type Screen,
} from "../lib/desk";

// Re-exported under the old name so the rest of the UI keeps the same type
// surface. Rust owns the derivation now (see desk-core `Screen`).
export type AppState = Screen;

// How long a press has to outlast before we flip the readout into its "Moving"
// state. A brief tap on Lower/Raise barely nudges the desk (if at all), so
// showing "Moving up/down" for it just makes the status line flicker. Releasing
// cancels the pending flip, so anything shorter than this never changes the UI.
const MOVE_DISPLAY_DEBOUNCE_MS = 200;

export function useDesk() {
  const [appState, setAppState] = useState<Screen>("connecting");
  const [connection, setConnection] = useState<ConnectionState>("connecting");
  const [heightCm, setHeightCm] = useState<number | null>(null);
  // null = not moving. The direction carries the moving bit, so there's no
  // separate `moving` flag.
  const [moveDirection, setMoveDirection] = useState<Direction | null>(null);
  const [scanResults, setScanResults] = useState<DeskInfo[]>([]);
  // address of the desk we're currently trying to connect to, sourced from the
  // `desk-connection` event payload. The row shown on the scan screen merges
  // this with the live scan results (and falls back to the remembered name).
  const [connectingAddress, setConnectingAddress] = useState<string | null>(
    null,
  );
  // name of the currently connected (or remembered) desk, surfaced in the UI
  const [deskName, setDeskName] = useState<string | null>(null);
  // Last bluetooth radio state seen; kept in a ref because the UI renders from
  // `appState` (Rust-owned screen) and only the off->on edge matters here, to
  // trigger a recovery boot.
  const lastBluetoothRef = useRef<"ready" | "off" | "unknown">("unknown");
  // Pending timer for flipping the readout into its "Moving" state. Held so a
  // release (or a `moving: false` motion event) can cancel a not-yet-shown move,
  // which is what debounces away the flicker from a quick tap.
  const moveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Update the displayed move direction, debouncing the *start* of a move so a
  // quick tap never shows "Moving". Stopping (and the boot path) takes effect
  // immediately and cancels any pending flip. This only smooths the visual
  // indicator; Rust still owns whether the desk actually moves.
  const setMoveDirectionDebounced = useCallback((next: Direction | null) => {
    if (moveTimerRef.current) {
      clearTimeout(moveTimerRef.current);
      moveTimerRef.current = null;
    }
    if (next === null) {
      setMoveDirection(null);
      return;
    }
    moveTimerRef.current = setTimeout(() => {
      moveTimerRef.current = null;
      setMoveDirection(next);
    }, MOVE_DISPLAY_DEBOUNCE_MS);
  }, []);

  // Apply the screen Rust chose for us on launch. The matching backend events
  // follow and keep us in sync from there on.
  const applyBoot = useCallback((b: BootState) => {
    setDeskName(b.name);
    setAppState(b.screen);
    lastBluetoothRef.current = b.screen === "bluetooth_off" ? "off" : "ready";
    switch (b.screen) {
      case "connected":
        setConnection("connected");
        setHeightCm(b.height_cm);
        // BootState carries only the moving bit, not the direction. The rare
        // case of a webview reload mid-hold defaults to "up" until the next
        // motion event lands; matches the prior default-arrow render.
        setMoveDirection(b.moving ? "up" : null);
        setConnectingAddress(null);
        break;
      case "connecting":
        setConnection("connecting");
        setConnectingAddress(b.address);
        break;
      case "bluetooth_off":
        setConnection("disconnected");
        setConnectingAddress(null);
        break;
      case "scanning":
        setConnection("disconnected");
        setConnectingAddress(null);
        setScanResults([]);
        break;
    }
  }, []);

  useEffect(() => {
    const pending: Promise<UnlistenFn>[] = [
      onHeight((e) => {
        setHeightCm(e.cm);
      }),
      onConnection((e) => {
        setConnection(e.state);
        if (e.name) setDeskName(e.name);
        setConnectingAddress(e.state === "connecting" ? e.address : null);
      }),
      onMotion((e) => {
        setMoveDirectionDebounced(e.moving ? e.direction : null);
      }),
      onDiscovered((d) => {
        setScanResults((prev) => {
          const i = prev.findIndex((x) => x.address === d.address);
          if (i === -1) return [...prev, d];
          const next = prev.slice();
          next[i] = d;
          return next;
        });
      }),
      onBluetooth(({ state }) => {
        // Rust handles dropping any live link before reporting `off`. We only
        // watch for the off->on edge here to trigger a recovery boot.
        const prev = lastBluetoothRef.current;
        lastBluetoothRef.current = state;
        if (state === "ready" && prev === "off") {
          desk
            .boot()
            .then(applyBoot)
            .catch(() => {});
        }
      }),
      onScreen((e) => {
        setAppState(e.screen);
      }),
    ];

    // Hand the whole startup decision to Rust. A duplicate call (React
    // StrictMode mounts this effect twice in dev) is deduped backend-side by
    // the boot guard, so it never kicks off a second connect.
    desk
      .boot()
      .then(applyBoot)
      .catch(() => {
        // boot itself failed (rare): fall back to a scan
        setConnection("disconnected");
        setAppState("scanning");
        desk.scanStart().catch(() => {});
      });

    return () => {
      if (moveTimerRef.current) clearTimeout(moveTimerRef.current);
      Promise.all(pending).then((fns) => {
        for (const f of fns) f();
      });
    };
  }, [applyBoot, setMoveDirectionDebounced]);

  // The row shown on the scan/connecting screen for the desk we're trying to
  // connect to. Built from the live scan list when available (so the rssi bars
  // are real) and from the remembered name otherwise. Returns null when no
  // connect attempt is in flight.
  const connectingTarget: DeskInfo | null = connectingAddress
    ? (scanResults.find((r) => r.address === connectingAddress) ?? {
        name: deskName ?? "Your desk",
        address: connectingAddress,
        rssi: null,
      })
    : null;

  // --- actions -------------------------------------------------------------

  const startScan = useCallback(async () => {
    setScanResults([]);
    setConnectingAddress(null);
    try {
      await desk.scanStart();
    } catch {
      setConnection("disconnected");
    }
  }, []);

  const connectTo = useCallback(
    async (target: DeskInfo) => {
      setConnectingAddress(target.address);
      setConnection("connecting"); // immediate feedback before the event lands
      // Rust serializes connects and stops the scan internally, so we don't
      // need to scanStop() first.
      const ok = await desk.connect(target.address, target.name);
      if (!ok) startScan();
    },
    [startScan],
  );

  const disconnect = useCallback(async () => {
    await desk.disconnect();
    startScan();
  }, [startScan]);

  const holdStart = useCallback(
    (dir: Direction) => {
      // Optimistic, but debounced: a quick tap is cancelled by the matching
      // release (`stop`) before the readout ever flips to "Moving".
      setMoveDirectionDebounced(dir);
      desk.moveStart(dir).catch(() => {});
    },
    [setMoveDirectionDebounced],
  );

  // Press-and-hold a preset. Unlike holdStart, we don't optimistically set a
  // direction: Rust derives it from current vs. target height (Rust owns the
  // decision), so we wait for the motion event rather than guessing here.
  const holdTarget = useCallback((targetCm: number) => {
    desk.moveToStart(targetCm).catch(() => {});
  }, []);

  const stop = useCallback(() => {
    // Release cancels a pending (not-yet-shown) move immediately, so a tap
    // shorter than the debounce window leaves the readout untouched. The
    // authoritative `moving: false` motion event follows and agrees.
    setMoveDirectionDebounced(null);
    desk.stop().catch(() => {});
  }, [setMoveDirectionDebounced]);

  const recheckBluetooth = useCallback(async () => {
    // Belt-and-braces fallback in case `watch_bluetooth` missed the toggle (or
    // the event stream is mid-restart). Re-asks Rust and lets it re-decide.
    const bt = await desk.bluetoothState().catch(() => "off" as const);
    if (bt === "ready") {
      lastBluetoothRef.current = "ready";
      desk
        .boot()
        .then(applyBoot)
        .catch(() => startScan());
    }
  }, [applyBoot, startScan]);

  return {
    appState,
    connection,
    heightCm,
    moveDirection,
    scanResults,
    connectingTarget,
    deskName,
    // actions
    connectTo,
    disconnect,
    holdStart,
    holdTarget,
    stop,
    recheckBluetooth,
    openBtSettings: desk.openBluetoothSettings,
  };
}
