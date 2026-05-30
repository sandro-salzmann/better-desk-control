import { Button as RACButton } from "react-aria-components";
import type { DeskInfo } from "../../lib/desk";
import { Bluetooth } from "lucide-react";
import { SignalBars } from "../atoms/SignalBars";

interface Props {
  desk: DeskInfo;
  onConnect: (d: DeskInfo) => void;
}

// A discovered desk in the scan list.
export function DeskRow({ desk, onConnect }: Props) {
  return (
    <RACButton
      onPress={() => onConnect(desk)}
      className={({ isFocusVisible }) =>
        `flex w-full cursor-pointer items-center gap-3 rounded-xl border border-line-strong bg-surface-1 px-4 py-3 text-left outline-none transition hover:border-bluetooth/50 hover:bg-bluetooth/8 ${
          isFocusVisible ? "ring-2 ring-accent/50" : ""
        }`
      }
    >
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
      <SignalBars rssi={desk.rssi} />
    </RACButton>
  );
}
