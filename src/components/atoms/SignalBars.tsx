const BAR_HEIGHTS = [5, 9, 13, 16];

function rssiLevel(rssi: number | null): number {
  if (rssi == null) return 2;
  if (rssi >= -55) return 4;
  if (rssi >= -67) return 3;
  if (rssi >= -80) return 2;
  return 1;
}

// Four-bar Bluetooth signal-strength meter.
export const SignalBars = ({ rssi }: { rssi: number | null }) => {
  const level = rssiLevel(rssi);
  return (
    <span className="flex h-4 items-end gap-1">
      {BAR_HEIGHTS.map((h, i) => (
        <i
          key={i}
          className={`w-0.75 rounded-xs bg-bluetooth ${i < level ? "opacity-100" : "opacity-35"}`}
          style={{ height: `${h}px` }}
        />
      ))}
    </span>
  );
};
