import { MoreHorizontal, Pencil, Save, Trash2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import {
  Input,
  Menu,
  MenuItem,
  MenuTrigger,
  Popover,
  Button as RACButton,
  TextField,
} from "react-aria-components";
import type { Preset } from "../../lib/presets";
import { formatHeight } from "../../lib/units";
import { Badge } from "../atoms/Badge";

interface Props {
  preset: Preset;
  isCurrent: boolean;
  connected: boolean;
  heightCm: number | null;
  onApply: (p: Preset) => void;
  onOverwrite: (id: string) => void;
  onRemove: (id: string) => void;
  onRename: (id: string, name: string) => void;
}

// A single preset card. The whole card body is the "move to" tap target; a
// single overflow menu in the top-right collects the secondary actions (save
// current height here, rename, delete). The overflow button sits above the
// card-wide tap layer with its own pointer target so near-misses route to the
// menu instead of triggering an unintended move.
export function PresetRow({
  preset,
  isCurrent,
  connected,
  heightCm,
  onApply,
  onOverwrite,
  onRemove,
  onRename,
}: Props) {
  const [renaming, setRenaming] = useState(false);
  const [draft, setDraft] = useState(preset.name);
  const inputRef = useRef<HTMLInputElement>(null);

  // keep the draft in sync with externally driven rename (e.g. the gallery)
  useEffect(() => {
    setDraft(preset.name);
  }, [preset.name]);

  // focus the rename field as soon as the menu enters rename mode
  useEffect(() => {
    if (renaming) inputRef.current?.focus();
  }, [renaming]);

  function commitName() {
    onRename(preset.id, draft.trim() || preset.name);
    setRenaming(false);
  }

  const tapDisabled = !connected || renaming;
  const overwriteLabel =
    heightCm != null
      ? `Save current height (${formatHeight(heightCm)} cm) here`
      : "Save current height here";

  return (
    <div
      className={`group relative flex items-center gap-3 rounded-2xl border py-4 pr-2 pl-4 transition ${
        isCurrent
          ? "border-accent/55 bg-accent/8"
          : "border-line-strong bg-surface-1 hover:border-line-hover hover:bg-surface-2"
      }`}
    >
      <RACButton
        aria-label={`Move to ${preset.name}`}
        isDisabled={tapDisabled}
        onPress={() => onApply(preset)}
        className={({ isFocusVisible }) =>
          `absolute inset-0 z-0 rounded-2xl outline-none ${
            tapDisabled ? "cursor-default" : "cursor-pointer"
          } ${isFocusVisible ? "ring-2 ring-inset ring-accent/50" : ""}`
        }
      />

      <div className="pointer-events-none relative z-10 flex w-full items-center gap-3">
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-2 text-base font-semibold text-fg">
            {renaming ? (
              <TextField
                aria-label="Preset name"
                value={draft}
                onChange={setDraft}
                className="pointer-events-auto"
              >
                <Input
                  ref={inputRef}
                  onFocus={(e) => e.target.select()}
                  onBlur={commitName}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") commitName();
                    if (e.key === "Escape") {
                      setDraft(preset.name);
                      setRenaming(false);
                    }
                  }}
                  className="-mx-2 -my-px w-[8.5em] rounded-md border border-line-strong bg-surface-0 px-2 py-px text-base font-semibold text-fg outline-none focused:border-accent/50"
                />
              </TextField>
            ) : (
              <span>{preset.name}</span>
            )}
            {isCurrent && <Badge>Current</Badge>}
          </span>
          <span
            className={`mt-1 block font-mono text-[19px] font-medium ${
              isCurrent ? "text-accent" : "text-fg"
            }`}
          >
            {formatHeight(preset.targetCm)}
            <span className="ml-1 font-sans text-xs text-fg-subtle">cm</span>
          </span>
        </span>

        <span className="pointer-events-auto shrink-0">
          <MenuTrigger>
            <RACButton
              aria-label={`More actions for ${preset.name}`}
              className={({ isFocusVisible }) =>
                `flex h-10 w-10 cursor-pointer items-center justify-center rounded-lg text-fg-subtle outline-none transition hover:bg-surface-3 hover:text-fg ${
                  isFocusVisible ? "ring-2 ring-accent/50" : ""
                }`
              }
            >
              <MoreHorizontal className="h-4 w-4" />
            </RACButton>
            <Popover
              placement="bottom end"
              offset={8}
              className="w-66 origin-top-right rounded-xl border border-line-strong bg-surface-2 p-1 text-left shadow-xl outline-none"
            >
              <Menu className="outline-none">
                <MenuItem
                  isDisabled={!connected || heightCm == null}
                  onAction={() => onOverwrite(preset.id)}
                  className={({ isDisabled, isFocused }) =>
                    `flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium outline-none transition [&_svg]:h-4 [&_svg]:w-4 ${
                      isDisabled
                        ? "cursor-not-allowed text-fg-subtle opacity-60"
                        : isFocused
                          ? "cursor-pointer bg-surface-3 text-fg"
                          : "cursor-pointer text-fg-muted"
                    }`
                  }
                >
                  <Save />
                  {overwriteLabel}
                </MenuItem>
                <MenuItem
                  onAction={() => {
                    setDraft(preset.name);
                    setRenaming(true);
                  }}
                  className={({ isFocused }) =>
                    `flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium outline-none transition [&_svg]:h-4 [&_svg]:w-4 ${
                      isFocused ? "bg-surface-3 text-fg" : "text-fg-muted"
                    }`
                  }
                >
                  <Pencil />
                  Rename
                </MenuItem>
                <MenuItem
                  onAction={() => onRemove(preset.id)}
                  className={({ isFocused }) =>
                    `flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium text-stop outline-none transition hover:bg-stop/12 [&_svg]:h-4 [&_svg]:w-4 ${
                      isFocused ? "bg-stop/12" : ""
                    }`
                  }
                >
                  <Trash2 />
                  Delete
                </MenuItem>
              </Menu>
            </Popover>
          </MenuTrigger>
        </span>
      </div>
    </div>
  );
}
