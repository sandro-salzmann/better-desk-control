import {
  Button as RACButton,
  Dialog,
  DialogTrigger,
  Popover,
} from "react-aria-components";
import { Power, Settings } from "lucide-react";
import { Button } from "../atoms/Button";

interface Props {
  connected: boolean;
  onDisconnect: () => void;
}

// The settings gear and its popover. DialogTrigger handles open state, focus
// management and outside-click dismissal for us.
export function SettingsMenu({ connected, onDisconnect }: Props) {
  return (
    <DialogTrigger>
      <Button square size="md" aria-label="Settings">
        <Settings />
      </Button>
      <Popover
        placement="bottom end"
        offset={8}
        className="w-62 origin-top-right rounded-xl border border-line-strong bg-surface-2 p-2 text-left shadow-xl outline-none"
      >
        <Dialog className="outline-none">
          {({ close }) => (
            <>
              {connected && (
                <RACButton
                  onPress={() => {
                    close();
                    onDisconnect();
                  }}
                  className="flex w-full cursor-pointer items-center gap-3 rounded-lg px-3 py-3 text-sm font-medium text-fg-muted outline-none transition hover:bg-surface-3 hover:text-fg focus-visible:bg-surface-3 [&_svg]:h-4 [&_svg]:w-4"
                >
                  <Power />
                  Disconnect desk
                </RACButton>
              )}
            </>
          )}
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}
