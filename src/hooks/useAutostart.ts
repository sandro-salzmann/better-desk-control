// "Launch at startup" toggle state. The OS owns the truth (a registry / login
// item entry), so we read it once on mount and write through on each toggle,
// mirroring the actual result back so the switch never drifts from reality.

import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { useEffect, useState } from "react";

export function useAutostart() {
  // null until the first read resolves, so the UI can hold off rendering a
  // possibly-wrong state.
  const [enabled, setEnabled] = useState<boolean | null>(null);

  useEffect(() => {
    isEnabled()
      .then(setEnabled)
      .catch(() => setEnabled(false));
  }, []);

  const toggle = async (next: boolean) => {
    setEnabled(next); // optimistic; reconciled below
    try {
      if (next) await enable();
      else await disable();
    } finally {
      setEnabled(await isEnabled().catch(() => !next));
    }
  };

  return { enabled, toggle };
}
