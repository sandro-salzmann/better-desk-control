import { useEffect, type RefObject } from "react";
import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
} from "@tauri-apps/api/window";

/**
 * Grows/shrinks the OS window to match the height of `ref`'s content, capped at
 * `maxFraction` of the monitor height. Past the cap the window stays put and the
 * app's scroll container (`h-full overflow-y-auto`) shows a scrollbar.
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
) {
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
        const height = Math.min(Math.ceil(el.offsetHeight), maxLogicalHeight);
        if (height === lastHeight) return; // avoid redundant resizes
        lastHeight = height;
        appWindow.setSize(new LogicalSize(logicalWidth, height)).catch(() => {});
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
}
