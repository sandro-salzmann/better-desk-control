import type { ConnectionState, Direction } from "../../lib/desk";
import { HeightReadout } from "../molecules/HeightReadout";
import { SettingsMenu } from "../molecules/SettingsMenu";

interface Props {
  heightCm: number | null;
  connection: ConnectionState;
  moveDirection: Direction | null;
  deskName: string | null;
  onDisconnect: () => void;
}

// App header: live height + status on the left; the connection indicator and
// settings gear on the right. The indicator closes the loop with the scan
// screen ("you connected to Desk 6420") without forcing the user to open the
// settings popover to remember which desk they're on.
export function Header({
  heightCm,
  connection,
  moveDirection,
  deskName,
  onDisconnect,
}: Props) {
  const connected = connection === "connected";
  return (
    <div className="flex items-start justify-between gap-3">
      <HeightReadout
        heightCm={heightCm}
        connection={connection}
        moveDirection={moveDirection}
      />
      <div className="mt-1 shrink-0">
        <SettingsMenu
          connected={connected}
          deskName={deskName}
          onDisconnect={onDisconnect}
        />
      </div>
    </div>
  );
}
