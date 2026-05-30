import type { ReactNode } from "react";

// Full-window state overlay (connecting / scanning / bluetooth-off). Frosted
// backdrop over the app; `top` aligns content to the top for the scan list.
export function OverlayShell({
  top = false,
  children,
}: {
  top?: boolean;
  children: ReactNode;
}) {
  return (
    <div
      className={`absolute inset-0 z-50 flex flex-col items-center overflow-y-auto bg-surface-0/55 px-6 text-center backdrop-blur-[14px] ${
        top ? "justify-start pt-7 pb-8" : "justify-center py-8"
      }`}
    >
      <div className="flex w-full max-w-100 flex-col items-center gap-2">
        {children}
      </div>
    </div>
  );
}
