import { Power, Settings } from "lucide-react";
import {
  Dialog,
  DialogTrigger,
  Popover,
  Button as RACButton,
} from "react-aria-components";
import { Button } from "../atoms/Button";

interface Props {
  connected: boolean;
  deskName: string | null;
  onDisconnect: () => void;
}

// The settings gear and its popover. DialogTrigger handles open state, focus
// management and outside-click dismissal for us.
export function SettingsMenu({ connected, deskName, onDisconnect }: Props) {
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
                <>
                  <div className="flex items-center gap-2 px-3 pt-2 pb-3">
                    <span className="relative grid h-2.5 w-2.5 place-items-center">
                      <span className="absolute inset-0 animate-ping rounded-full bg-accent/55" />
                      <span className="relative h-2 w-2 rounded-full bg-accent" />
                    </span>
                    <span className="min-w-0 flex-1 truncate text-sm font-semibold text-fg">
                      {deskName ?? "Connected"}
                    </span>
                  </div>
                  <div className="mb-1 border-t border-line" />
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
                </>
              )}
            </>
          )}
        </Dialog>
      </Popover>
    </DialogTrigger>
  );
}
