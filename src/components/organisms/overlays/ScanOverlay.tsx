import type { DeskInfo } from "../../../lib/desk";
import { Bluetooth } from "lucide-react";
import { Spinner } from "../../atoms/Spinner";
import { DeskRow } from "../../molecules/DeskRow";
import { OverlayShell } from "./OverlayShell";

interface Props {
  results: DeskInfo[];
  scanning: boolean;
  // the desk we're trying to connect to, shown as a marked row even before it's
  // rediscovered by a scan (e.g. during auto-reconnect)
  connecting: DeskInfo | null;
  onConnect: (d: DeskInfo) => void;
}

export function ScanOverlay({
  results,
  scanning,
  connecting,
  onConnect,
}: Props) {
  // make sure the desk we're connecting to is in the list (and only once)
  const rows =
    connecting && !results.some((r) => r.address === connecting.address)
      ? [connecting, ...results]
      : results;

  const subtitle = connecting
    ? "Connecting…"
    : rows.length
      ? "Tap a desk to connect"
      : "Scanning for nearby desks…";

  return (
    <OverlayShell top>
      <div className="mb-1 flex items-center gap-2">
        <div className="text-lg font-semibold tracking-[-0.2px] text-fg">
          Available desks
        </div>
        {scanning && <Spinner size="sm" tone="bluetooth" />}
      </div>
      <div className="text-sm font-medium text-fg-muted">{subtitle}</div>

      {rows.length ? (
        <div className="mt-4 mb-1 flex w-full flex-col gap-2">
          {rows.map((d) => (
            <DeskRow
              key={d.address}
              desk={d}
              onConnect={onConnect}
              connecting={!!connecting && d.address === connecting.address}
            />
          ))}
        </div>
      ) : (
        <div className="flex flex-col items-center gap-2 pt-6 pb-1 text-fg-muted">
          <div className="relative mb-2 grid h-12 w-12 place-items-center text-bluetooth [&_svg]:relative [&_svg]:z-10 [&_svg]:h-6 [&_svg]:w-6">
            <span className="absolute inset-0 animate-ring-ping rounded-full border-[1.5px] border-bluetooth opacity-0" />
            <span className="absolute inset-0 animate-ring-ping rounded-full border-[1.5px] border-bluetooth opacity-0 [animation-delay:1s]" />
            <Bluetooth />
          </div>
          <div className="text-sm font-semibold text-fg">
            No desks found yet
          </div>
          <div className="text-xs font-medium">
            Make sure your desk is powered on
          </div>
        </div>
      )}

      <div className="mt-4 flex w-full flex-col gap-2.5 border-t border-line pt-4 text-start text-xs leading-[1.6] text-fg-muted">
        <p>
          <b className="font-semibold text-fg">Won't connect?</b>
          <br /> A desk pairs with one device at a time, so disconnect it from
          the phone app or other computers first.
        </p>
        <p>
          <b className="font-semibold text-fg">Missing a desk?</b>
          <br /> Hold its pairing button for 5&nbsp;s to make it discoverable.
        </p>
      </div>
    </OverlayShell>
  );
}
