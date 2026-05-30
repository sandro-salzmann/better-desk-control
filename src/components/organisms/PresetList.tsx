import { type Preset } from "../../lib/presets";
import { PresetRow } from "../molecules/PresetRow";
import { AddPresetRow } from "../molecules/AddPresetRow";

interface Props {
  presets: Preset[];
  currentId: string | null;
  connected: boolean;
  canAdd: boolean;
  onApply: (p: Preset) => void;
  onOverwrite: (id: string) => void;
  onRemove: (id: string) => void;
  onRename: (id: string, name: string) => void;
  onAdd: () => void;
}

export function PresetList({
  presets,
  currentId,
  connected,
  canAdd,
  onApply,
  onOverwrite,
  onRemove,
  onRename,
  onAdd,
}: Props) {
  return (
    <>
      <div className="mx-1 flex items-center justify-between">
        <span className="font-mono text-[10px] font-medium uppercase tracking-[2px] text-fg-subtle">
          Presets
        </span>
      </div>
      <div className="flex flex-col gap-3">
        {presets.map((p) => (
          <PresetRow
            key={p.id}
            preset={p}
            isCurrent={p.id === currentId}
            connected={connected}
            onApply={onApply}
            onOverwrite={onOverwrite}
            onRemove={onRemove}
            onRename={onRename}
          />
        ))}
        <AddPresetRow canAdd={canAdd} onAdd={onAdd} />
      </div>
    </>
  );
}
