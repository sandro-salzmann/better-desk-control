import { ArrowDown, ArrowUp, Check } from "lucide-react";
import type { MoveIntent } from "../../hooks/useDesk";
import type { ConnectionState } from "../../lib/desk";
import { formatHeight } from "../../lib/units";

interface Props {
  heightCm: number | null;
  connection: ConnectionState;
  moving: boolean;
  moveIntent: MoveIntent | null;
  atPresetName: string | null;
}

function StatusLine({
  connection,
  moving,
  moveIntent,
  atPresetName,
}: Pick<Props, "connection" | "moving" | "moveIntent" | "atPresetName">) {
  const base =
    "mt-3 flex items-center gap-2 text-xs font-medium [&_svg]:h-4 [&_svg]:w-4";

  if (connection !== "connected") {
    return <div className={`${base} text-fg-subtle`}>Not connected</div>;
  }
  if (moving) {
    const down = moveIntent?.dir === "down";
    const Arrow = down ? ArrowDown : ArrowUp;
    return (
      <div className={`${base} ${down ? "text-lower" : "text-accent"}`}>
        <Arrow />
        {moveIntent?.name ? `Moving to ${moveIntent.name}…` : "Adjusting…"}
      </div>
    );
  }
  if (atPresetName) {
    return (
      <div className={`${base} text-accent`}>
        <Check />
        At preset · {atPresetName}
      </div>
    );
  }
  return <div className={`${base} text-fg-subtle`}>Ready</div>;
}

// The large live-height number plus its status line.
export function HeightReadout({
  heightCm,
  connection,
  moving,
  moveIntent,
  atPresetName,
}: Props) {
  const value = formatHeight(connection === "connected" ? heightCm : null);

  return (
    <div className="min-w-0">
      <div className="font-mono text-[84px] font-bold leading-[0.9] tracking-[-5px] text-fg">
        {value}
        <span className="ml-2 font-sans text-[21px] font-semibold tracking-normal text-fg-muted">
          cm
        </span>
      </div>
      <StatusLine
        connection={connection}
        moving={moving}
        moveIntent={moveIntent}
        atPresetName={atPresetName}
      />
    </div>
  );
}
