import type { ReactNode } from "react";

// Small uppercase caption that sits above a section of the main screen. Sets
// the group apart through typography and whitespace rather than a card frame.
export function SectionLabel({ children }: { children: ReactNode }) {
  return (
    <span className="text-[11px] font-semibold uppercase tracking-[0.12em] text-fg-muted">
      {children}
    </span>
  );
}
