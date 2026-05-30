import { useEffect, useRef } from "react";
import { FineAdjust } from "./components/organisms/FineAdjust";
import { Header } from "./components/organisms/Header";
import { BluetoothOffOverlay } from "./components/organisms/overlays/BluetoothOffOverlay";
import { ScanOverlay } from "./components/organisms/overlays/ScanOverlay";
import { PresetList } from "./components/organisms/PresetList";
import { useDesk } from "./hooks/useDesk";
import { useFitWindowHeight } from "./hooks/useFitWindowHeight";
import { usePresets } from "./lib/presets";

function App() {
  const {
    appState,
    connection,
    heightCm,
    moveDirection,
    scanResults,
    connectingTarget,
    deskName,
    connectTo,
    disconnect,
    holdStart,
    holdTarget,
    stop,
    recheckBluetooth,
    openBtSettings,
  } = useDesk();
  const { presets, add, overwrite, remove, rename } = usePresets();

  // auto-fit the OS window to the content
  const contentRef = useRef<HTMLDivElement>(null);
  useFitWindowHeight(contentRef);

  const connected = connection === "connected";

  // re-check Bluetooth when the user returns from the OS settings
  useEffect(() => {
    if (appState !== "bluetooth_off") return;
    const onFocus = () => recheckBluetooth();
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [appState, recheckBluetooth]);

  // an overlay covers the app in every state but "connected"; mark the content
  // behind it inert so keyboard/AT can't reach the buttons under the backdrop
  const overlayActive = appState !== "connected";

  return (
    <div className="relative h-full overflow-hidden bg-surface-0">
      <div className="h-full overflow-y-auto" inert={overlayActive}>
        <div
          ref={contentRef}
          className="mx-auto flex max-w-100 flex-col gap-5 p-6"
        >
          <Header
            heightCm={heightCm}
            connection={connection}
            moveDirection={moveDirection}
            deskName={deskName}
            onDisconnect={disconnect}
          />
          <PresetList
            presets={presets}
            connected={connected}
            canAdd={connected && heightCm != null}
            heightCm={heightCm}
            onMoveStart={(preset) => holdTarget(preset.targetCm)}
            onMoveEnd={stop}
            onOverwrite={(id) => heightCm != null && overwrite(id, heightCm)}
            onRemove={remove}
            onRename={rename}
            onAdd={() => heightCm != null && add(heightCm)}
          />
          {/* hairline that splits the two flat zones without re-introducing a card */}
          <div aria-hidden className="h-px w-full bg-white/6" />
          <FineAdjust connected={connected} onHold={holdStart} onStop={stop} />
        </div>
      </div>

      {(appState === "scanning" || appState === "connecting") && (
        <ScanOverlay
          results={scanResults}
          scanning={appState === "scanning"}
          connecting={connectingTarget}
          onConnect={connectTo}
        />
      )}
      {appState === "bluetooth_off" && (
        <BluetoothOffOverlay
          onEnable={() => openBtSettings().catch(() => {})}
        />
      )}
    </div>
  );
}

export default App;
