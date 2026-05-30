import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
} from "@tauri-apps/api/window";
import { type RefObject, useEffect, useState } from "react";

/**
 * Grows/shrinks the OS window to match the height of `ref`'s content, capped at
 * `maxFraction` of the monitor height. Past the cap the window stays put and the
 * caller is expected to let its scroll container show a scrollbar.
 *
 * Returns whether the content is currently capped (i.e. taller than the window).
 * Below the cap the window always grows to fit, so a scrollbar is never needed;
 * the caller should keep overflow hidden until `capped` is true. Otherwise the
 * scrollbar flashes for the one frame between content growing and the window
 * catching up.
 *
 * Why measure a content element instead of the window: the window *is* the
 * webview viewport, so `100dvh` / `body.scrollHeight` equal the window height,
 * useless for deciding how tall the window should be. We measure the natural
 * height of the inner content node, which is independent of the viewport, and
 * the `75dvh` requirement becomes "75% of the monitor", resolved here in JS.
 */
export function useFitWindowHeight(
  ref: RefObject<HTMLElement | null>,
  maxFraction = 0.75,
): boolean {
  const [capped, setCapped] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const appWindow = getCurrentWindow();
    let maxLogicalHeight = Infinity;
    let logicalWidth = 0;
    let lastHeight = -1;
    let raf = 0;
    let disposed = false;

    const apply = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        if (disposed || !logicalWidth) return;
        const natural = Math.ceil(el.offsetHeight);
        const height = Math.min(natural, maxLogicalHeight);
        setCapped(natural > maxLogicalHeight);
        if (height === lastHeight) return; // avoid redundant resizes
        lastHeight = height;
        appWindow
          .setSize(new LogicalSize(logicalWidth, height))
          .catch(() => {});
      });
    };

    const ro = new ResizeObserver(apply);

    (async () => {
      const [monitor, inner, scale] = await Promise.all([
        currentMonitor(),
        appWindow.innerSize(),
        appWindow.scaleFactor(),
      ]);
      if (disposed) return;
      // keep the (fixed, non-resizable) width; only height tracks content
      logicalWidth = Math.round(inner.width / scale);
      if (monitor) {
        maxLogicalHeight = Math.floor(
          (monitor.size.height * maxFraction) / monitor.scaleFactor,
        );
      }
      ro.observe(el);
      apply();
    })();

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  }, [ref, maxFraction]);

  return capped;
}
