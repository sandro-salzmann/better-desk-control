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

interface Props {
  preset: Preset;
  connected: boolean;
  heightCm: number | null;
  onMoveStart: (preset: Preset) => void;
  onMoveEnd: () => void;
  onOverwrite: (id: string) => void;
  onRemove: (id: string) => void;
  onRename: (id: string, name: string) => void;
}

// A single preset card. Holding the card drives the desk to the preset height
// and releasing stops it (Rust picks the direction and the stop point). The
// overflow menu in the top-right collects the secondary actions (save current
// height here, rename, delete).
export function PresetRow({
  preset,
  connected,
  heightCm,
  onMoveStart,
  onMoveEnd,
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

  const overwriteLabel =
    heightCm != null
      ? `Save current height (${formatHeight(heightCm)} cm) here`
      : "Save current height here";

  // Shared card chrome. The whole card is the hold-to-move button (except while
  // renaming, when it hosts the text field instead); `pr-14` reserves room for
  // the overflow menu that floats over the top-right corner.
  const cardBase =
    "flex w-full items-center gap-3 rounded-2xl border border-line-strong py-4 pr-14 pl-4 transition";

  return (
    <div className="group relative">
      {renaming ? (
        <div className={`${cardBase} bg-surface-1`}>
          <span className="min-w-0 flex-1">
            <span className="flex items-center gap-2 text-base font-semibold text-fg">
              <TextField
                aria-label="Preset name"
                value={draft}
                onChange={setDraft}
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
                  className="-mx-2 -my-px w-[8.5em] rounded-md bg-surface-0 px-2 py-px text-base font-semibold text-fg outline-none ring-1 ring-line-strong ring-inset focused:ring-accent/50"
                />
              </TextField>
            </span>
            <span className="mt-1 block font-mono text-[19px] font-medium text-fg">
              {formatHeight(preset.targetCm)}
              <span className="ml-1 font-sans text-xs text-fg-subtle">cm</span>
            </span>
          </span>
        </div>
      ) : (
        <RACButton
          aria-label={`Hold to move desk to ${preset.name}, ${formatHeight(preset.targetCm)} cm`}
          isDisabled={!connected}
          onPressStart={() => onMoveStart(preset)}
          onPressEnd={onMoveEnd}
          className={({ isFocusVisible, isPressed, isDisabled }) =>
            `${cardBase} text-left outline-none ${
              isDisabled
                ? "cursor-not-allowed bg-surface-1 opacity-60"
                : isPressed
                  ? "cursor-pointer bg-surface-3"
                  : "cursor-pointer bg-surface-1 hover:bg-surface-2"
            } ${isFocusVisible ? "ring-2 ring-accent/50" : ""}`
          }
        >
          <span className="min-w-0 flex-1">
            <span className="flex items-center gap-2 text-base font-semibold text-fg">
              {preset.name}
            </span>
            <span className="mt-1 block font-mono text-[19px] font-medium text-fg">
              {formatHeight(preset.targetCm)}
              <span className="ml-1 font-sans text-xs text-fg-subtle">cm</span>
            </span>
          </span>
        </RACButton>
      )}

      {/* Floats over the card button (absolute + later in the DOM), so a tap on
          the menu lands here and never starts a hold-to-move on the card. */}
      <span className="absolute top-1/2 right-2 -translate-y-1/2">
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
  );
}
