import { ArrowDown, ArrowUp } from "lucide-react";
import { Button } from "../atoms/Button";
import { SectionLabel } from "../atoms/SectionLabel";

interface Props {
  connected: boolean;
  onHold: (dir: "up" | "down") => void;
  onStop: () => void;
}

export function FineAdjust({ connected, onHold, onStop }: Props) {
  return (
    <section className="flex flex-col gap-3">
      <div className="flex items-baseline justify-between">
        <SectionLabel>Fine adjust</SectionLabel>
        <span className="text-xs text-fg-subtle">
          Hold to move continuously
        </span>
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
    </section>
  );
}
