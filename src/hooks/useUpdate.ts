// Self-update UI state. Rust drives the whole flow on launch (check -> download
// -> install) and reports it via `update-*` events; this hook only mirrors
// those events and exposes the install action for the "Restart now" button.

import type { UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import {
  installUpdate,
  onUpdateAvailable,
  onUpdateProgress,
  onUpdateReady,
} from "../lib/desk";

export type UpdateStatus = "idle" | "downloading" | "ready";

export function useUpdate() {
  const [status, setStatus] = useState<UpdateStatus>("idle");
  const [version, setVersion] = useState<string | null>(null);
  const [progress, setProgress] = useState(0); // 0..1; 0 when size is unknown
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    const pending: Promise<UnlistenFn>[] = [
      onUpdateAvailable((e) => {
        setVersion(e.version);
        setProgress(0);
        setDismissed(false);
        setStatus("downloading");
      }),
      onUpdateProgress((e) => {
        if (e.total) setProgress(Math.min(1, e.downloaded / e.total));
      }),
      onUpdateReady((e) => {
        setVersion(e.version);
        setStatus("ready");
      }),
    ];
    return () => {
      Promise.all(pending).then((fns) => {
        for (const f of fns) f();
      });
    };
  }, []);

  return {
    status,
    version,
    progress,
    // The banner shows while downloading and once ready, until dismissed.
    visible: status !== "idle" && !dismissed,
    dismiss: () => setDismissed(true),
    // Fire-and-forget: on success the backend relaunches the app, so this
    // promise never meaningfully resolves on the happy path.
    install: () => installUpdate().catch(() => {}),
  };
}
