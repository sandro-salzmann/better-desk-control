import type { ReactNode } from "react";

// A small accent pill, e.g. the "Current" marker on the active preset.
export const Badge = ({ children }: { children: ReactNode }) => (
  <span className="rounded-md border border-accent/30 bg-accent/12 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide text-accent">
    {children}
  </span>
);
