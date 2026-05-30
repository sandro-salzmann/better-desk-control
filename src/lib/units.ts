// Height display helpers. The frontend works purely in cm; desk-core (Rust)
// owns the raw<->cm conversion, so there is no raw math here.

/// Format a cm value for display (one decimal place), or a placeholder.
export function formatHeight(cm: number | null): string {
  if (cm == null || Number.isNaN(cm)) return "--.-";
  return cm.toFixed(1);
}
