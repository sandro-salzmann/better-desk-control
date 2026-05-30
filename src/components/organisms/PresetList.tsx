import type { Preset } from "../../lib/presets";
import { SectionLabel } from "../atoms/SectionLabel";
import { AddPresetRow } from "../molecules/AddPresetRow";
import { PresetRow } from "../molecules/PresetRow";

interface Props {
  presets: Preset[];
  connected: boolean;
  canAdd: boolean;
  heightCm: number | null;
  onMoveStart: (preset: Preset) => void;
  onMoveEnd: () => void;
  onOverwrite: (id: string) => void;
  onRemove: (id: string) => void;
  onRename: (id: string, name: string) => void;
  onAdd: () => void;
}

export function PresetList({
  presets,
  connected,
  canAdd,
  heightCm,
  onMoveStart,
  onMoveEnd,
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
            connected={connected}
            heightCm={heightCm}
            onMoveStart={onMoveStart}
            onMoveEnd={onMoveEnd}
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
