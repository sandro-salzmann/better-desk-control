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

export interface MoveIntent {
  // preset the user tapped, or null for a manual hold
  name: string | null;
  // direction the desk is being driven, reported by Rust via desk-motion. Null
  // for the brief window between tapping a preset and the motion event landing.
  dir: Direction | null;
}

export function useDesk() {
  const [appState, setAppState] = useState<Screen>("connecting");
  const [connection, setConnection] = useState<ConnectionState>("connecting");
  const [heightCm, setHeightCm] = useState<number | null>(null);
  const [moving, setMoving] = useState(false);
  const [moveIntent, setMoveIntent] = useState<MoveIntent | null>(null);
  const [scanResults, setScanResults] = useState<DeskInfo[]>([]);
  // address of the desk we're currently trying to connect to, sourced from the
  // `desk-connection` event payload. The row shown on the scan screen merges
  // this with the live scan results (and falls back to the remembered name).
  const [connectingAddress, setConnectingAddress] = useState<string | null>(
    null,
  );
  // arrival tolerance (cm) for "at this preset", reported by desk-core
  const [toleranceCm, setToleranceCm] = useState<number | null>(null);
  // name of the currently connected (or remembered) desk, surfaced in the UI
  const [deskName, setDeskName] = useState<string | null>(null);
  // Last bluetooth radio state seen; kept in a ref because the UI renders from
  // `appState` (Rust-owned screen) and only the off->on edge matters here, to
  // trigger a recovery boot.
  const lastBluetoothRef = useRef<"ready" | "off" | "unknown">("unknown");

  // Apply the screen Rust chose for us on launch. The matching backend events
  // follow and keep us in sync from there on.
  const applyBoot = useCallback((b: BootState) => {
    setToleranceCm(b.arrive_tolerance_cm);
    setDeskName(b.name);
    setAppState(b.screen);
    lastBluetoothRef.current = b.screen === "bluetooth_off" ? "off" : "ready";
    switch (b.screen) {
      case "connected":
        setConnection("connected");
        setHeightCm(b.height_cm);
        setMoving(b.moving);
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
        setMoving(e.moving);
        if (e.moving) {
          // Keep the preset name a preceding moveToPreset stashed; the motion
          // event itself doesn't carry it.
          setMoveIntent((prev) => ({
            name: prev?.name ?? null,
            dir: e.direction,
          }));
        } else {
          setMoveIntent(null);
        }
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
      Promise.all(pending).then((fns) => {
        for (const f of fns) f();
      });
    };
  }, [applyBoot]);

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

  const moveToPreset = useCallback((name: string, targetCm: number) => {
    // Stash the preset name optimistically; the motion event fills in `dir`
    // (Rust derives it from the authoritative current height). The status line
    // only reads MoveIntent once `moving` is true, so the null-dir window is
    // never rendered.
    setMoveIntent({ name, dir: null });
    desk.moveToHeight(targetCm).catch(() => setMoveIntent(null));
  }, []);

  const holdStart = useCallback((dir: Direction) => {
    setMoveIntent({ name: null, dir });
    desk.moveStart(dir).catch(() => setMoveIntent(null));
  }, []);

  const stop = useCallback(() => {
    desk.stop().catch(() => {});
  }, []);

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
    moving,
    moveIntent,
    scanResults,
    connectingTarget,
    toleranceCm,
    deskName,
    // actions
    connectTo,
    disconnect,
    moveToPreset,
    holdStart,
    stop,
    recheckBluetooth,
    openBtSettings: desk.openBluetoothSettings,
  };
}
