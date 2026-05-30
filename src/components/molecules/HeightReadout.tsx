import { ArrowDown, ArrowUp, BluetoothOff } from "lucide-react";
import type { ConnectionState, Direction } from "../../lib/desk";
import { formatHeight } from "../../lib/units";

interface Props {
  heightCm: number | null;
  connection: ConnectionState;
  moveDirection: Direction | null;
}

// A pulsing dot for the resting "Ready" state, so it stands apart visually from
// the gray disconnected state instead of being just another piece of text.
function StatusDot({ tone }: { tone: "accent" | "subtle" }) {
  return (
    <span className="relative grid h-2.5 w-2.5 place-items-center">
      {tone === "accent" && (
        <span className="absolute inset-0 animate-ping rounded-full bg-accent/55" />
      )}
      <span
        className={`relative h-2 w-2 rounded-full ${
          tone === "accent" ? "bg-accent" : "bg-fg-subtle"
        }`}
      />
    </span>
  );
}

function StatusLine({
  connection,
  moveDirection,
}: Pick<Props, "connection" | "moveDirection">) {
  const base =
    "mt-3 flex items-center gap-2 text-xs font-medium [&_svg]:h-4 [&_svg]:w-4";

  if (connection !== "connected") {
    return (
      <div className={`${base} text-fg-subtle`}>
        <BluetoothOff />
        Disconnected
      </div>
    );
  }
  if (moveDirection) {
    const down = moveDirection === "down";
    const Arrow = down ? ArrowDown : ArrowUp;
    const tone = down ? "text-lower" : "text-accent";
    return (
      <div className={`${base} ${tone}`}>
        <Arrow />
        {down ? "Moving down" : "Moving up"}
      </div>
    );
  }
  return (
    <div className={`${base} text-accent`}>
      <StatusDot tone="accent" />
      Ready
    </div>
  );
}

// The large live-height number plus its status line.
export function HeightReadout({ heightCm, connection, moveDirection }: Props) {
  const value = formatHeight(connection === "connected" ? heightCm : null);

  return (
    <div className="min-w-0">
      <div className="font-mono text-[84px] font-bold leading-[0.9] tracking-[-5px] text-fg">
        {value}
        <span className="ml-2 font-sans text-[21px] font-semibold tracking-normal text-fg-muted">
          cm
        </span>
      </div>
      <StatusLine connection={connection} moveDirection={moveDirection} />
    </div>
  );
}
