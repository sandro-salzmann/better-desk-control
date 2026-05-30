import { Plus } from "lucide-react";
import { Button as RACButton } from "react-aria-components";

interface Props {
  canAdd: boolean;
  onAdd: () => void;
}

// The dashed "Add preset" slot at the end of the list.
export function AddPresetRow({ canAdd, onAdd }: Props) {
  return (
    <RACButton
      isDisabled={!canAdd}
      onPress={onAdd}
      className={({ isDisabled, isFocusVisible }) =>
        `group flex w-full items-center gap-4 rounded-2xl border border-dashed border-line-hover bg-transparent py-4 pr-4 pl-4 text-left outline-none transition ${
          isDisabled
            ? "pointer-events-none cursor-not-allowed opacity-45"
            : "cursor-pointer hover:border-accent/50 hover:bg-accent/5"
        } ${isFocusVisible ? "ring-2 ring-accent/50" : ""}`
      }
    >
      <span className="grid h-11 w-11 shrink-0 place-items-center rounded-xl border border-line-strong text-fg-subtle transition group-hover:text-accent [&_svg]:h-6 [&_svg]:w-6">
        <Plus />
      </span>
      <span className="flex-1 text-base font-medium text-fg-subtle transition group-hover:text-fg">
        Add preset
      </span>
    </RACButton>
  );
}
