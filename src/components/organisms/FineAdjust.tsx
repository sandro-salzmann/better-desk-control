import { ArrowDown, ArrowUp } from "lucide-react";
import { Button } from "../atoms/Button";

interface Props {
  connected: boolean;
  onHold: (dir: "up" | "down") => void;
  onStop: () => void;
}

export function FineAdjust({ connected, onHold, onStop }: Props) {
  return (
    <>
      <div className="mx-1 flex items-center justify-between">
        <span className="font-mono text-[10px] font-medium uppercase tracking-[2px] text-fg-subtle">
          Fine adjust
        </span>
        <span className="text-xs text-fg-subtle">Hold to move</span>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Button
          tone="lower"
          fullWidth
          isDisabled={!connected}
          onPressStart={() => onHold("down")}
          onPressEnd={onStop}
        >
          <ArrowDown />
          Lower
        </Button>
        <Button
          tone="accent"
          fullWidth
          isDisabled={!connected}
          onPressStart={() => onHold("up")}
          onPressEnd={onStop}
        >
          Raise
          <ArrowUp />
        </Button>
      </div>
    </>
  );
}
