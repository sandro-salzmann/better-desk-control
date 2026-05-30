import { Bluetooth } from "lucide-react";
import { Button as RACButton } from "react-aria-components";
import type { DeskInfo } from "../../lib/desk";
import { SignalBars } from "../atoms/SignalBars";
import { Spinner } from "../atoms/Spinner";

interface Props {
  desk: DeskInfo;
  onConnect: (d: DeskInfo) => void;
  // we're currently trying to connect to this desk: mark it and lock the row
  connecting?: boolean;
}

// A discovered desk in the scan list.
export function DeskRow({ desk, onConnect, connecting = false }: Props) {
  return (
    <RACButton
      isDisabled={connecting}
      onPress={() => onConnect(desk)}
      className={({ isFocusVisible }) =>
        `relative flex w-full items-center gap-3 rounded-xl border px-4 py-3 text-left outline-none transition ${
          connecting
            ? "cursor-default border-bluetooth/50 bg-bluetooth/8"
            : "cursor-pointer border-line-strong bg-surface-1 hover:border-bluetooth/50 hover:bg-bluetooth/8"
        } ${isFocusVisible ? "ring-2 ring-accent/50" : ""}`
      }
    >
      {/* pulsing highlight while connecting */}
      {connecting && (
        <span className="pointer-events-none absolute inset-0 animate-pulse rounded-xl ring-2 ring-bluetooth/40" />
      )}
      <span className="relative grid h-9 w-9 shrink-0 place-items-center rounded-full border border-line-strong bg-surface-2 text-bluetooth [&_svg]:relative [&_svg]:z-10 [&_svg]:h-4 [&_svg]:w-4">
        <span className="absolute inset-0 animate-bt-pulse rounded-full border-[1.5px] border-bluetooth opacity-0" />
        <span className="absolute inset-0 animate-bt-pulse rounded-full border-[1.5px] border-bluetooth opacity-0 [animation-delay:1.2s]" />
        <Bluetooth />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm font-semibold text-fg">
          {desk.name}
        </span>
      </span>
      {connecting ? (
        <span className="flex shrink-0 items-center gap-1.5 rounded-md border border-bluetooth/30 bg-bluetooth/12 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide text-bluetooth">
          Trying to connect…
          <Spinner size="xs" tone="bluetooth" />
        </span>
      ) : (
        <SignalBars rssi={desk.rssi} />
      )}
    </RACButton>
  );
}
