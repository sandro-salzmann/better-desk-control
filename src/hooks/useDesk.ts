// The app's desk/connection state. Rust decides *what to do* on launch and on a
// Bluetooth recovery (reconnect to the saved desk vs. scan) in the `desk_boot`
// command; this hook just applies that decision, subscribes to the backend
// events, and exposes actions + a derived `appState` that selects which
// screen/overlay to show.

import { useCallback, useEffect, useRef, useState } from "react";
import {
  desk,
  onBluetooth,
  onConnection,
  onDiscovered,
  onHeight,
  onMotion,
  type BootState,
  type ConnectionState,
  type DeskInfo,
} from "../lib/desk";
import type { UnlistenFn } from "@tauri-apps/api/event";

export type BtState = "checking" | "ready" | "off";

export type AppState =
  | "connecting"
  | "connected"
  | "bluetooth_off"
  | "scanning";

export interface MoveIntent {
  name: string | null; // preset name, or null for a manual hold
  dir: "up" | "down";
}

export function useDesk() {
  const [connection, setConnection] = useState<ConnectionState>("connecting");
  const [heightCm, setHeightCm] = useState<number | null>(null);
  const [moving, setMoving] = useState(false);
  const [moveIntent, setMoveIntent] = useState<MoveIntent | null>(null);
  const [scanResults, setScanResults] = useState<DeskInfo[]>([]);
  // the desk we're currently trying to connect to (auto-reconnect or a tapped
  // scan result), shown as a "trying to connect" row on the scan screen
  const [connectingTarget, setConnectingTarget] = useState<DeskInfo | null>(
    null,
  );
  const [btState, setBtState] = useState<BtState>("checking");
  // arrival tolerance (cm) for "at this preset", reported by desk-core
  const [toleranceCm, setToleranceCm] = useState<number | null>(null);

  const heightCmRef = useRef<number | null>(null);
  heightCmRef.current = heightCm;

  // lets the live Bluetooth listener tell a real off->on recovery from the
  // initial "ready" report without re-subscribing.
  const btStateRef = useRef<BtState>("checking");
  btStateRef.current = btState;

  // Apply the screen Rust chose for us (on launch, or re-deciding after a
  // Bluetooth recovery). The matching backend events follow and keep us in sync.
  const applyBoot = useCallback((b: BootState) => {
    setToleranceCm(b.arrive_tolerance_cm);
    switch (b.screen) {
      case "connected":
        setConnection("connected");
        setBtState("ready");
        setHeightCm(b.height_cm);
        setMoving(b.moving);
        setConnectingTarget(null);
        break;
      case "connecting":
        setConnection("connecting");
        setBtState("ready");
        // list the remembered desk as the row we're trying to connect to
        setConnectingTarget({
          name: b.name ?? "Your desk",
          address: b.address ?? "",
          rssi: null,
        });
        break;
      case "bluetooth_off":
        setBtState("off");
        setConnection("disconnected");
        setConnectingTarget(null);
        break;
      case "scanning":
        setBtState("ready");
        setConnection("disconnected");
        setConnectingTarget(null);
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
        // the attempt resolved (connected) or fell through (disconnected): the
        // "trying to connect" row is no longer current
        if (e.state !== "connecting") setConnectingTarget(null);
      }),
      onMotion((e) => {
        setMoving(e.moving);
        if (!e.moving) setMoveIntent(null);
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
      // live adapter state: the radio was toggled while the app is open
      onBluetooth(({ state }) => {
        const prev = btStateRef.current;
        if (state === "off") {
          // the radio is gone: tear down the link but keep the saved desk so a
          // recovery can reconnect to it.
          setBtState("off");
          setConnection("disconnected");
          setHeightCm(null);
          desk.drop().catch(() => {});
        } else {
          setBtState("ready");
          // only act on a real recovery; the initial "ready" is boot's job
          if (prev === "off") desk.boot().then(applyBoot).catch(() => {});
        }
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
        setBtState("ready");
        setConnection("disconnected");
        desk.scanStart().catch(() => {});
      });

    return () => {
      Promise.all(pending).then((fns) => fns.forEach((f) => f()));
    };
  }, [applyBoot]);

  // --- actions -------------------------------------------------------------

  const startScan = useCallback(async () => {
    setScanResults([]);
    setConnectingTarget(null);
    try {
      await desk.scanStart();
    } catch {
      setBtState("off");
      setConnection("disconnected");
    }
  }, []);

  const connectTo = useCallback(
    async (target: DeskInfo) => {
      setConnectingTarget(target);
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
    const cur = heightCmRef.current ?? 0;
    setMoveIntent({ name, dir: targetCm >= cur ? "up" : "down" });
    desk.moveToHeight(targetCm).catch(() => setMoveIntent(null));
  }, []);

  const holdStart = useCallback((dir: "up" | "down") => {
    setMoveIntent({ name: null, dir });
    desk.moveStart(dir).catch(() => setMoveIntent(null));
  }, []);

  const stop = useCallback(() => {
    desk.stop().catch(() => {});
  }, []);

  const recheckBluetooth = useCallback(async () => {
    setBtState("checking");
    const bt = await desk.bluetoothState().catch(() => "off" as const);
    if (bt === "off") {
      setBtState("off");
      return;
    }
    setBtState("ready");
    // Bluetooth is back: let Rust re-decide (reconnect to the saved desk or scan)
    desk.boot().then(applyBoot).catch(() => startScan());
  }, [applyBoot, startScan]);

  const appState: AppState =
    connection === "connected"
      ? "connected"
      : btState === "off"
        ? "bluetooth_off"
        : connection === "connecting"
          ? "connecting"
          : "scanning";

  return {
    appState,
    connection,
    heightCm,
    moving,
    moveIntent,
    scanResults,
    connectingTarget,
    toleranceCm,
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
