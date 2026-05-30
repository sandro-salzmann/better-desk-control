import { Save, Trash2 } from "lucide-react";
import { useState } from "react";
import { Input, Button as RACButton, TextField } from "react-aria-components";
import type { Preset } from "../../lib/presets";
import { formatHeight } from "../../lib/units";
import { Badge } from "../atoms/Badge";
import { Button } from "../atoms/Button";
import { Tooltip } from "../atoms/Tooltip";

interface Props {
  preset: Preset;
  isCurrent: boolean;
  connected: boolean;
  onApply: (p: Preset) => void;
  onOverwrite: (id: string) => void;
  onRemove: (id: string) => void;
  onRename: (id: string, name: string) => void;
}

// A single preset card. The full card is a primary "move to" action (an
// absolutely-positioned button behind the content); the name, overwrite and
// delete controls layer on top with their own pointer targets.
export function PresetRow({
  preset,
  isCurrent,
  connected,
  onApply,
  onOverwrite,
  onRemove,
  onRename,
}: Props) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(preset.name);
  const value = formatHeight(preset.targetCm);

  function commit() {
    onRename(preset.id, draft.trim() || preset.name);
    setEditing(false);
  }

  return (
    <div
      className={`relative flex items-center gap-4 rounded-2xl border py-4 pr-4 pl-4 transition ${
        isCurrent
          ? "border-accent/55 bg-accent/8"
          : "border-line-strong bg-surface-1 hover:border-line-hover hover:bg-surface-2"
      }`}
    >
      {/* Primary action: covers the whole card, sits behind the content. */}
      <RACButton
        aria-label={`Move to ${preset.name}`}
        isDisabled={!connected || editing}
        onPress={() => onApply(preset)}
        className={({ isFocusVisible }) =>
          `absolute inset-0 z-0 rounded-2xl outline-none ${
            connected && !editing ? "cursor-pointer" : "cursor-default"
          } ${isFocusVisible ? "ring-2 ring-inset ring-accent/50" : ""}`
        }
      />

      <div className="pointer-events-none relative z-10 flex w-full items-center gap-4">
        <span className="min-w-0 flex-1">
          <span className="flex items-center gap-2 text-base font-semibold">
            {editing ? (
              <TextField
                aria-label="Preset name"
                value={draft}
                onChange={setDraft}
                className="pointer-events-auto"
              >
                <Input
                  autoFocus
                  onFocus={(e) => e.target.select()}
                  onBlur={commit}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") commit();
                    if (e.key === "Escape") {
                      setDraft(preset.name);
                      setEditing(false);
                    }
                  }}
                  className="-mx-2 -my-px w-[8.5em] rounded-md border border-line-strong bg-surface-0 px-2 py-px text-base font-semibold text-fg outline-none focused:border-accent/50"
                />
              </TextField>
            ) : (
              <RACButton
                onPress={() => {
                  setDraft(preset.name);
                  setEditing(true);
                }}
                className="pointer-events-auto -mx-2 -my-px cursor-text rounded-md border border-transparent px-2 py-px text-left text-fg outline-none transition hover:bg-white/6 focus-visible:bg-white/6"
              >
                {preset.name}
              </RACButton>
            )}
            {isCurrent && <Badge>Current</Badge>}
          </span>
          <span
            className={`mt-1 block font-mono text-[19px] font-medium ${
              isCurrent ? "text-accent" : "text-fg"
            }`}
          >
            {value}
            <span className="ml-1 font-sans text-xs text-fg-subtle">cm</span>
          </span>
        </span>

        <span className="pointer-events-auto flex shrink-0 gap-2">
          <Tooltip content="Overwrite with current height">
            <Button
              square
              tone="accent"
              size="sm"
              isDisabled={!connected}
              aria-label="Overwrite with current height"
              onPress={() => onOverwrite(preset.id)}
            >
              <Save />
            </Button>
          </Tooltip>
          <Tooltip content="Delete preset">
            <Button
              square
              tone="stop"
              size="sm"
              aria-label="Delete preset"
              onPress={() => onRemove(preset.id)}
            >
              <Trash2 />
            </Button>
          </Tooltip>
        </span>
      </div>
    </div>
  );
}
