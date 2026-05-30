import type { DeskInfo } from "../../lib/desk";
import { Spinner } from "../atoms/Spinner";
import { DeskRowBody } from "./DeskRow";

interface Props {
  desk: DeskInfo;
}

// The "trying to connect" row shown above the scan list while a connect is in
// flight (auto-reconnect or a tapped scan result). It's a sibling of `DeskRow`
// rather than a flag on it so neither component has to branch on intent.
export function ConnectingDeskRow({ desk }: Props) {
  return (
    <div className="relative flex w-full cursor-default items-center gap-3 rounded-xl border border-bluetooth/50 bg-bluetooth/8 px-4 py-3 text-left">
      <span className="pointer-events-none absolute inset-0 animate-pulse rounded-xl ring-2 ring-bluetooth/40" />
      <DeskRowBody
        name={desk.name}
        trailing={
          <span className="flex shrink-0 items-center gap-1.5 rounded-md border border-bluetooth/30 bg-bluetooth/12 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide text-bluetooth">
            Trying to connect…
            <Spinner size="xs" tone="bluetooth" />
          </span>
        }
      />
    </div>
  );
}
