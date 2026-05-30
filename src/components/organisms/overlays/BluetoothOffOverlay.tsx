import { Bluetooth, BluetoothOff } from "lucide-react";
import { Button } from "../../atoms/Button";
import { OverlayShell } from "./OverlayShell";

export function BluetoothOffOverlay({ onEnable }: { onEnable: () => void }) {
  return (
    <OverlayShell>
      <div className="mb-4 grid h-14 w-14 place-items-center rounded-2xl border border-line-strong bg-surface-1 text-stop [&_svg]:h-7 [&_svg]:w-7">
        <BluetoothOff />
      </div>
      <div className="text-lg font-semibold tracking-[-0.2px] text-fg">
        Bluetooth unavailable
      </div>
      <div className="text-sm font-medium text-fg-muted">
        Turn on Bluetooth to find and control your desk
      </div>
      <Button
        variant="primary"
        tone="bluetooth"
        onPress={onEnable}
        className="mt-4 rounded-xl"
      >
        <Bluetooth className="h-5 w-5" />
        Open Bluetooth settings
      </Button>
    </OverlayShell>
  );
}
