import type { Preset } from "../../lib/presets";
import { SectionLabel } from "../atoms/SectionLabel";
import { AddPresetRow } from "../molecules/AddPresetRow";
import { PresetRow } from "../molecules/PresetRow";

interface Props {
  presets: Preset[];
  currentId: string | null;
  connected: boolean;
  canAdd: boolean;
  heightCm: number | null;
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
  heightCm,
  onApply,
  onOverwrite,
  onRemove,
  onRename,
  onAdd,
}: Props) {
  return (
    <section className="flex flex-col gap-3">
      <SectionLabel>Presets</SectionLabel>
      <div className="flex flex-col gap-3">
        {presets.map((p) => (
          <PresetRow
            key={p.id}
            preset={p}
            isCurrent={p.id === currentId}
            connected={connected}
            heightCm={heightCm}
            onApply={onApply}
            onOverwrite={onOverwrite}
            onRemove={onRemove}
            onRename={onRename}
          />
        ))}
      </div>
      <AddPresetRow canAdd={canAdd} heightCm={heightCm} onAdd={onAdd} />
    </section>
  );
}
