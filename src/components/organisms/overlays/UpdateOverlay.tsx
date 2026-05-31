import { RotateCw } from "lucide-react";
import type { UpdateStatus } from "../../../hooks/useUpdate";
import { Button } from "../../atoms/Button";
import { Spinner } from "../../atoms/Spinner";
import { OverlayShell } from "./OverlayShell";

interface Props {
  status: UpdateStatus;
  version: string | null;
  progress: number; // 0..1
  onRestart: () => void;
}

// Covers the app while a newer release downloads and, once the bytes are staged,
// prompts a restart. Both states render through the shared OverlayShell, matching
// the scan/Bluetooth gates.
export function UpdateOverlay({ status, version, progress, onRestart }: Props) {
  if (status === "idle") return null;

  if (status === "downloading") {
    const pct = Math.round(progress * 100);
    return (
      <OverlayShell>
        <Spinner size="lg" tone="accent" />
        <div className="mt-5 text-lg font-semibold tracking-[-0.2px] text-fg">
          {`Downloading update${version ? ` ${version}` : ""}…`}
        </div>
        <div className="mt-1 text-sm font-medium text-fg-muted">{pct}%</div>
        <div className="mt-5 h-1 w-60 overflow-hidden rounded-full bg-surface-3">
          <div
            className="h-full rounded-full bg-accent transition-[width]"
            style={{ width: `${pct}%` }}
          />
        </div>
      </OverlayShell>
    );
  }

  return (
    <OverlayShell>
      <div className="text-lg font-semibold tracking-[-0.2px] text-fg">
        {`Update ${version ?? ""} ready`.trim()}
      </div>
      <div className="mt-1 text-sm font-medium text-fg-muted">
        Restart to finish installing
      </div>
      <Button variant="primary" onPress={onRestart} className="mt-5">
        <RotateCw size={14} />
        Restart now
      </Button>
    </OverlayShell>
  );
}
