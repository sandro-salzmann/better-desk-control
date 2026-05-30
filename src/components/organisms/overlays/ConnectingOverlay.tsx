import { Spinner } from "../../atoms/Spinner";
import { OverlayShell } from "./OverlayShell";

export function ConnectingOverlay({ name }: { name: string | null }) {
  return (
    <OverlayShell>
      <Spinner size="lg" tone="accent" className="mb-4" />
      <div className="text-lg font-semibold tracking-[-0.2px] text-fg">
        Connecting…
      </div>
      <div className="text-sm font-medium text-fg-muted">
        Pairing with {name ?? "your desk"} over Bluetooth
      </div>
    </OverlayShell>
  );
}
