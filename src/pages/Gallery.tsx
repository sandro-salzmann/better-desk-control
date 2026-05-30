// Component gallery: a living catalogue of every atom and molecule.
//
// Not part of the desk app. It is rendered standalone by `components.html` →
// `src/components.tsx`, so it never ships inside the real UI. To view it, run
// the dev server and open `/components.html`, or build and open
// `dist/components.html`.
//
// Everything here renders with mock data so it works with no desk / Tauri
// backend connected.

import { useState, type ReactNode } from "react";
import { Bluetooth, Save, Settings, Trash2 } from "lucide-react";

import { Badge } from "../components/atoms/Badge";
import { Button } from "../components/atoms/Button";
import { SignalBars } from "../components/atoms/SignalBars";
import { Spinner } from "../components/atoms/Spinner";
import { Tooltip } from "../components/atoms/Tooltip";

import { AddPresetRow } from "../components/molecules/AddPresetRow";
import { DeskRow } from "../components/molecules/DeskRow";
import { HeightReadout } from "../components/molecules/HeightReadout";
import { PresetRow } from "../components/molecules/PresetRow";
import { SettingsMenu } from "../components/molecules/SettingsMenu";

import type { DeskInfo } from "../lib/desk";
import type { Preset } from "../lib/presets";

// ── Layout helpers ──────────────────────────────────────────────────────────

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mb-12">
      <h2 className="mb-5 border-b border-line pb-2 text-xs font-semibold uppercase tracking-[2px] text-fg-subtle">
        {title}
      </h2>
      <div className="flex flex-col gap-7">{children}</div>
    </section>
  );
}

// A labelled specimen of one component (or one set of variants).
function Specimen({
  label,
  children,
  align = "items-center",
}: {
  label: string;
  children: ReactNode;
  align?: string;
}) {
  return (
    <div className="flex flex-col gap-3">
      <span className="font-mono text-xs text-fg-subtle">{label}</span>
      <div className={`flex flex-wrap gap-3 ${align}`}>{children}</div>
    </div>
  );
}

// ── Mock data ───────────────────────────────────────────────────────────────

const BUTTON_VARIANTS = ["primary", "secondary", "ghost"] as const;
const BUTTON_TONES = ["neutral", "accent", "lower", "bluetooth", "stop"] as const;

const mockDesks: DeskInfo[] = [
  { name: "Desk 7F2A", address: "AA:BB:CC:DD:EE:01", rssi: -48 },
  { name: "Standing Desk Pro", address: "AA:BB:CC:DD:EE:02", rssi: -72 },
  { name: "Office Desk", address: "AA:BB:CC:DD:EE:03", rssi: -91 },
];

// ── Interactive demos (own local state so they actually work) ────────────────

function PresetRowDemo() {
  const [presets, setPresets] = useState<Preset[]>([
    { id: "a", name: "Sit", targetCm: 77 },
    { id: "b", name: "Stand", targetCm: 120 },
  ]);
  const rename = (id: string, name: string) =>
    setPresets((p) => p.map((x) => (x.id === id ? { ...x, name } : x)));

  return (
    <div className="flex max-w-100 flex-col gap-3">
      {presets.map((p, i) => (
        <PresetRow
          key={p.id}
          preset={p}
          isCurrent={i === 0}
          connected
          onApply={() => {}}
          onOverwrite={() => {}}
          onRemove={(id) => setPresets((ps) => ps.filter((x) => x.id !== id))}
          onRename={rename}
        />
      ))}
    </div>
  );
}

function SettingsMenuDemo() {
  return <SettingsMenu connected onDisconnect={() => {}} />;
}

// ── Page ────────────────────────────────────────────────────────────────────

export function Gallery() {
  return (
    <div className="h-full overflow-y-auto bg-surface-0 text-fg">
      <div className="mx-auto max-w-3xl px-6 py-10">
        <header className="mb-12 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold tracking-tight">
              Component gallery
            </h1>
            <p className="mt-1 text-sm text-fg-muted">
              Every atom & molecule, rendered with mock data.
            </p>
          </div>
          <Button variant="secondary" onPress={() => (window.location.href = "/")}>
            ← Back to app
          </Button>
        </header>

        {/* ── ATOMS ─────────────────────────────────────────────────────── */}
        <h1 className="mb-6 text-sm font-semibold uppercase tracking-[3px] text-accent">
          Atoms
        </h1>

        <Section title="Button: variant × tone">
          {BUTTON_VARIANTS.map((variant) => (
            <Specimen key={variant} label={variant}>
              {BUTTON_TONES.map((tone) => (
                <Button key={tone} variant={variant} tone={tone}>
                  {tone}
                </Button>
              ))}
            </Specimen>
          ))}
          <Specimen label="size: sm / md / lg">
            <Button size="sm">Small</Button>
            <Button size="md">Medium</Button>
            <Button size="lg">Large</Button>
          </Specimen>
          <Specimen label="square / disabled / fullWidth">
            <Button square>
              <Settings />
            </Button>
            <Button isDisabled>Disabled</Button>
            <div className="w-full">
              <Button fullWidth variant="primary" tone="lower">
                Full width
              </Button>
            </div>
          </Specimen>
        </Section>

        <Section title="Button: icon-only (square)">
          <Specimen label="tone: neutral / accent / bluetooth / stop">
            <Button square aria-label="Settings">
              <Settings />
            </Button>
            <Button square tone="accent" aria-label="Save">
              <Save />
            </Button>
            <Button square tone="bluetooth" aria-label="Bluetooth">
              <Bluetooth />
            </Button>
            <Button square tone="stop" aria-label="Delete">
              <Trash2 />
            </Button>
          </Specimen>
          <Specimen label="size: sm / md · disabled">
            <Button square size="sm" aria-label="Settings small">
              <Settings />
            </Button>
            <Button square size="md" aria-label="Settings medium">
              <Settings />
            </Button>
            <Button square isDisabled aria-label="Disabled">
              <Settings />
            </Button>
          </Specimen>
        </Section>

        <Section title="Badge">
          <Specimen label="accent pill">
            <Badge>Current</Badge>
            <Badge>New</Badge>
          </Specimen>
        </Section>

        <Section title="Spinner">
          <Specimen label="size: lg / sm · tone: accent / bluetooth">
            <Spinner size="lg" tone="accent" />
            <Spinner size="lg" tone="bluetooth" />
            <Spinner size="sm" tone="accent" />
            <Spinner size="sm" tone="bluetooth" />
          </Specimen>
        </Section>

        <Section title="Tooltip">
          <Specimen label="hover / focus a control">
            <Tooltip content="I appear on hover">
              <Button>Hover me</Button>
            </Tooltip>
            <Tooltip content="Delete preset" placement="right">
              <Button square tone="stop" aria-label="Delete">
                <Trash2 />
              </Button>
            </Tooltip>
          </Specimen>
        </Section>

        <Section title="SignalBars">
          <Specimen label="signal strength (-48 / -60 / -75 / -90 / null)">
            <SignalBars rssi={-48} />
            <SignalBars rssi={-60} />
            <SignalBars rssi={-75} />
            <SignalBars rssi={-90} />
            <SignalBars rssi={null} />
          </Specimen>
        </Section>

        {/* ── MOLECULES ─────────────────────────────────────────────────── */}
        <h1 className="mb-6 mt-4 text-sm font-semibold uppercase tracking-[3px] text-accent">
          Molecules
        </h1>

        <Section title="HeightReadout">
          <Specimen label="connected / moving / disconnected" align="items-start">
            <div className="rounded-2xl border border-line-strong bg-surface-1 p-5">
              <HeightReadout
                heightCm={88.5}
                connection="connected"
                moving={false}
                moveIntent={null}
                atPresetName="Stand"
              />
            </div>
            <div className="rounded-2xl border border-line-strong bg-surface-1 p-5">
              <HeightReadout
                heightCm={101.2}
                connection="connected"
                moving
                moveIntent={{ name: "Stand", dir: "up" }}
                atPresetName={null}
              />
            </div>
            <div className="rounded-2xl border border-line-strong bg-surface-1 p-5">
              <HeightReadout
                heightCm={null}
                connection="disconnected"
                moving={false}
                moveIntent={null}
                atPresetName={null}
              />
            </div>
          </Specimen>
        </Section>

        <Section title="DeskRow">
          <Specimen label="scan results (strong → weak signal)" align="items-stretch">
            <div className="flex w-full max-w-100 flex-col gap-3">
              {mockDesks.map((desk) => (
                <DeskRow key={desk.address} desk={desk} onConnect={() => {}} />
              ))}
            </div>
          </Specimen>
        </Section>

        <Section title="SettingsMenu">
          <Specimen label="gear → popover (interactive)">
            <SettingsMenuDemo />
          </Specimen>
        </Section>

        <Section title="PresetRow">
          <Specimen
            label="current + normal · rename / delete are live"
            align="items-stretch"
          >
            <PresetRowDemo />
          </Specimen>
        </Section>

        <Section title="AddPresetRow">
          <Specimen label="canAdd: true / false" align="items-stretch">
            <div className="flex w-full max-w-100 flex-col gap-3">
              <AddPresetRow canAdd onAdd={() => {}} />
              <AddPresetRow canAdd={false} onAdd={() => {}} />
            </div>
          </Specimen>
        </Section>

        <div className="h-10" />
      </div>
    </div>
  );
}
