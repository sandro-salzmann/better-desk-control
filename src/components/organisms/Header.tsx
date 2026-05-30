import type { ConnectionState } from "../../lib/desk";
import type { MoveIntent } from "../../hooks/useDesk";
import { HeightReadout } from "../molecules/HeightReadout";
import { SettingsMenu } from "../molecules/SettingsMenu";

interface Props {
  heightCm: number | null;
  connection: ConnectionState;
  moving: boolean;
  moveIntent: MoveIntent | null;
  atPresetName: string | null;
  onDisconnect: () => void;
}

// App header: live height + status on the left, settings gear on the right.
export function Header({
  heightCm,
  connection,
  moving,
  moveIntent,
  atPresetName,
  onDisconnect,
}: Props) {
  return (
    <div className="flex items-start justify-between">
      <HeightReadout
        heightCm={heightCm}
        connection={connection}
        moving={moving}
        moveIntent={moveIntent}
        atPresetName={atPresetName}
      />
      <div className="mt-1 shrink-0">
        <SettingsMenu
          connected={connection === "connected"}
          onDisconnect={onDisconnect}
        />
      </div>
    </div>
  );
}
