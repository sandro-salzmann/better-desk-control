// The app's desk/connection state machine: subscribes to the backend events,
// runs the boot/auto-reconnect flow, and exposes actions + a derived `appState`
// that selects which screen/overlay to show.

import { useCallback, useEffect, useRef, useState } from "react";
import {
  desk,
  onBluetooth,
  onConnection,
  onDiscovered,
  onHeight,
  onMotion,
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
  const [deskName, setDeskName] = useState<string | null>(null);
  const [pendingName, setPendingName] = useState<string | null>(null);
  const [heightCm, setHeightCm] = useState<number | null>(null);
  const [moving, setMoving] = useState(false);
  const [moveIntent, setMoveIntent] = useState<MoveIntent | null>(null);
  const [scanResults, setScanResults] = useState<DeskInfo[]>([]);
  const [scanning, setScanning] = useState(false);
  const [btState, setBtState] = useState<BtState>("checking");
  // arrival tolerance (cm) for "at this preset", reported by desk-core
  const [toleranceCm, setToleranceCm] = useState<number | null>(null);

  const heightCmRef = useRef<number | null>(null);
  heightCmRef.current = heightCm;

  // lets the live Bluetooth listener tell a real off->on recovery from the
  // initial "ready" report (which boot already handles) without re-subscribing.
  const btStateRef = useRef<BtState>("checking");
  btStateRef.current = btState;

  const startScan = useCallback(async () => {
    setScanResults([]);
    setScanning(true);
    try {
      await desk.scanStart();
    } catch {
      setScanning(false);
      setBtState("off");
      setConnection("disconnected");
    }
  }, []);

  // try the saved desk, falling back to a scan. Shared by boot and recovery.
  const reconnectOrScan = useCallback(async () => {
    const ok = await desk.connectSaved().catch(() => false);
    if (!ok) {
      setConnection("disconnected");
      startScan();
    }
  }, [startScan]);

  // boot: snapshot -> bluetooth check -> auto-reconnect -> scan
  const boot = useCallback(async () => {
    try {
      const snap = await desk.snapshot();
      setToleranceCm(snap.arrive_tolerance_cm);
      if (snap.connected) {
        setConnection("connected");
        setHeightCm(snap.height_cm);
        setMoving(snap.moving);
        return;
      }
      const bt = await desk.bluetoothState();
      if (bt === "off") {
        setBtState("off");
        setConnection("disconnected");
        return;
      }
      setBtState("ready");
      await reconnectOrScan();
    } catch {
      setConnection("disconnected");
      startScan();
    }
  }, [reconnectOrScan, startScan]);

  useEffect(() => {
    const pending: Promise<UnlistenFn>[] = [
      onHeight((e) => {
        setHeightCm(e.cm);
      }),
      onConnection((e) => {
        setConnection(e.state);
        if (e.name) setDeskName(e.name);
        if (e.state === "connected") {
          setPendingName(null);
          setScanning(false);
        }
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
          // the radio is gone: tear down any scan/connection so the overlay shows
          setBtState("off");
          setConnection("disconnected");
          setScanning(false);
          setHeightCm(null);
          desk.scanStop().catch(() => {});
          // also drop the backend connection so the two sides stay in sync
          desk.disconnect().catch(() => {});
        } else {
          setBtState("ready");
          // only act on a real recovery; the initial "ready" is boot's job
          if (prev === "off") reconnectOrScan();
        }
      }),
    ];

    boot();

    return () => {
      Promise.all(pending).then((fns) => fns.forEach((f) => f()));
      desk.scanStop().catch(() => {});
    };
  }, [boot, reconnectOrScan]);

  // --- actions -------------------------------------------------------------

  const connectTo = useCallback(
    async (target: DeskInfo) => {
      setPendingName(target.name);
      setScanning(false);
      await desk.scanStop().catch(() => {});
      const ok = await desk.connect(target.address);
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
    const bt = await desk.bluetoothState().catch(() => "off");
    if (bt === "off") {
      setBtState("off");
      return;
    }
    setBtState("ready");
    startScan();
  }, [startScan]);

  const appState: AppState =
    connection === "connected"
      ? "connected"
      : connection === "connecting"
        ? "connecting"
        : btState === "off"
          ? "bluetooth_off"
          : "scanning";

  return {
    appState,
    connection,
    deskName,
    pendingName,
    heightCm,
    moving,
    moveIntent,
    scanResults,
    scanning,
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
