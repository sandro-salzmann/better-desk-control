import { useCallback, useEffect, useMemo, useRef } from "react";
import { useDesk } from "./hooks/useDesk";
import { useFitWindowHeight } from "./hooks/useFitWindowHeight";
import { usePresets, type Preset } from "./lib/presets";
import { Header } from "./components/organisms/Header";
import { PresetList } from "./components/organisms/PresetList";
import { FineAdjust } from "./components/organisms/FineAdjust";
import { Button } from "./components/atoms/Button";
import { ScanOverlay } from "./components/organisms/overlays/ScanOverlay";
import { ConnectingOverlay } from "./components/organisms/overlays/ConnectingOverlay";
import { BluetoothOffOverlay } from "./components/organisms/overlays/BluetoothOffOverlay";

function App() {
  const {
    appState,
    connection,
    deskName,
    pendingName,
    heightCm,
    moving,
    moveIntent,
    scanResults,
    scanning,
    toleranceCm,
    connectTo,
    disconnect,
    moveToPreset,
    holdStart,
    stop,
    recheckBluetooth,
    openBtSettings,
  } = useDesk();
  const { presets, add, overwrite, remove, rename } = usePresets();

  // auto-fit the OS window to the content
  const contentRef = useRef<HTMLDivElement>(null);
  useFitWindowHeight(contentRef);

  const connected = connection === "connected";

  // which preset (if any) matches the current resting height
  const currentPreset = useMemo(() => {
    if (!connected || moving || heightCm == null || toleranceCm == null)
      return null;
    return (
      presets.find((p) => Math.abs(p.targetCm - heightCm) <= toleranceCm) ??
      null
    );
  }, [presets, heightCm, connected, moving, toleranceCm]);

  const currentId = currentPreset?.id ?? null;
  const atPresetName = currentPreset?.name ?? null;

  const applyPreset = useCallback(
    (p: Preset) => moveToPreset(p.name, p.targetCm),
    [moveToPreset],
  );

  // re-check Bluetooth when the user returns from the OS settings
  useEffect(() => {
    if (appState !== "bluetooth_off") return;
    const onFocus = () => recheckBluetooth();
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [appState, recheckBluetooth]);

  const onEnableBluetooth = () => {
    openBtSettings().catch(() => {});
  };

  // an overlay covers the app in every state but "connected"; mark the content
  // behind it inert so keyboard/AT can't reach the buttons under the backdrop
  const overlayActive = appState !== "connected";

  return (
    <div className="relative h-full overflow-hidden bg-surface-0">
      <div className="h-full overflow-y-auto" inert={overlayActive}>
        <div ref={contentRef} className="mx-auto flex max-w-100 flex-col gap-3 p-6">
          <Header
            heightCm={heightCm}
            connection={connection}
            moving={moving}
            moveIntent={moveIntent}
            atPresetName={atPresetName}
            onDisconnect={disconnect}
          />
          <PresetList
            presets={presets}
            currentId={currentId}
            connected={connected}
            canAdd={connected && heightCm != null && currentId == null}
            onApply={applyPreset}
            onOverwrite={(id) => heightCm != null && overwrite(id, heightCm)}
            onRemove={remove}
            onRename={rename}
            onAdd={() => heightCm != null && add(heightCm)}
          />
          <FineAdjust connected={connected} onHold={holdStart} onStop={stop} />
          <Button
            variant="primary"
            tone="stop"
            size="lg"
            fullWidth
            isDisabled={!moving}
            onPress={stop}
          >
            <span className="h-3 w-3 rounded-sm bg-current" />
            STOP
          </Button>
        </div>
      </div>

      {appState === "connecting" && (
        <ConnectingOverlay name={pendingName ?? deskName} />
      )}
      {appState === "scanning" && (
        <ScanOverlay
          results={scanResults}
          scanning={scanning}
          onConnect={connectTo}
        />
      )}
      {appState === "bluetooth_off" && (
        <BluetoothOffOverlay onEnable={onEnableBluetooth} />
      )}
    </div>
  );
}

export default App;
