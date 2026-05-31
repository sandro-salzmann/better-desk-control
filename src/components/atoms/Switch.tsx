import type { ReactNode } from "react";
import {
  Switch as RACSwitch,
  type SwitchProps as RACSwitchProps,
} from "react-aria-components";

interface Props extends Omit<RACSwitchProps, "children"> {
  children: ReactNode;
}

// A labelled on/off toggle. The whole row is the hit target; the track turns
// accent and the thumb slides right while selected.
export function Switch({ children, ...props }: Props) {
  return (
    <RACSwitch
      {...props}
      className="group flex w-full cursor-pointer items-center justify-between gap-3 rounded-lg px-3 py-3 text-sm font-medium text-fg-muted outline-none transition hover:bg-surface-3 hover:text-fg disabled:cursor-default disabled:opacity-40"
    >
      <span>{children}</span>
      <span className="flex h-5 w-9 shrink-0 items-center rounded-full bg-surface-3 px-0.5 transition-colors group-data-[selected]:bg-accent group-data-[focus-visible]:ring-2 group-data-[focus-visible]:ring-accent/50">
        <span className="h-4 w-4 rounded-full bg-fg shadow-sm transition-transform group-data-[selected]:translate-x-4" />
      </span>
    </RACSwitch>
  );
}
