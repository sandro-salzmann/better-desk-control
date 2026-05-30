import { Plus } from "lucide-react";
import { Button as RACButton } from "react-aria-components";
import { formatHeight } from "../../lib/units";

interface Props {
  canAdd: boolean;
  heightCm: number | null;
  onAdd: () => void;
}

// A quiet text button that sits flush under the preset list. No surface, no
// dashed placeholder: just a row that spells out the height that will be
// saved, so the action is concrete instead of an abstract "add".
export function AddPresetRow({ canAdd, heightCm, onAdd }: Props) {
  const label =
    canAdd && heightCm != null
      ? `Save ${formatHeight(heightCm)} cm as new preset`
      : "Save current height as new preset";
  return (
    <RACButton
      isDisabled={!canAdd}
      onPress={onAdd}
      className={({ isDisabled, isFocusVisible }) =>
        `mt-1 inline-flex w-fit items-center gap-1.5 self-start rounded-md px-1.5 py-1 text-sm font-medium outline-none transition [&_svg]:h-4 [&_svg]:w-4 ${
          isDisabled
            ? "cursor-not-allowed text-fg-subtle opacity-60"
            : "cursor-pointer text-fg-muted hover:text-fg"
        } ${isFocusVisible ? "ring-2 ring-accent/50" : ""}`
      }
    >
      <Plus />
      {label}
    </RACButton>
  );
}
