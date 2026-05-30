// App-managed presets, persisted via the Tauri store plugin. A preset is just a
// named target height; tapping it asks the backend to drive there.

import { useCallback, useEffect, useRef, useState } from "react";
import { load, type Store } from "@tauri-apps/plugin-store";

export interface Preset {
  id: string;
  name: string;
  targetCm: number;
}

const STORE_FILE = "presets.json";
const KEY = "presets";

// A cheap unique id without pulling in a dependency.
let idSeq = 0;
function newId(): string {
  idSeq += 1;
  return `p${Date.now().toString(36)}${idSeq}`;
}

export function usePresets() {
  const [presets, setPresets] = useState<Preset[]>([]);
  const storeRef = useRef<Store | null>(null);
  // gates persistence: we must not write before the on-disk state has loaded,
  // or the initial empty array would clobber it.
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let active = true;
    (async () => {
      const store = await load(STORE_FILE, { defaults: {}, autoSave: false });
      const saved = await store.get<Preset[]>(KEY);
      if (!active) return;
      storeRef.current = store;
      if (Array.isArray(saved)) setPresets(saved);
      setReady(true);
    })();
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!ready) return;
    const store = storeRef.current;
    if (!store) return;
    store
      .set(KEY, presets)
      .then(() => store.save())
      .catch(() => {
        /* ignore serialization / disk errors */
      });
  }, [presets, ready]);

  const add = useCallback((heightCm: number) => {
    setPresets((prev) => [
      ...prev,
      { id: newId(), name: `Preset ${prev.length + 1}`, targetCm: heightCm },
    ]);
  }, []);

  const overwrite = useCallback((id: string, heightCm: number) => {
    setPresets((prev) =>
      prev.map((p) => (p.id === id ? { ...p, targetCm: heightCm } : p)),
    );
  }, []);

  const remove = useCallback((id: string) => {
    setPresets((prev) => prev.filter((p) => p.id !== id));
  }, []);

  const rename = useCallback((id: string, name: string) => {
    setPresets((prev) =>
      prev.map((p) => (p.id === id ? { ...p, name } : p)),
    );
  }, []);

  return { presets, add, overwrite, remove, rename };
}
