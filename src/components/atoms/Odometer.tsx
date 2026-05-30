import { useEffect, useState } from "react";

// How many digit cells to render above and below the current one. The reel only
// ever steps by the shortest distance to the next digit (<= 5), so this window
// always covers the full travel even if updates stack up.
const RADIUS = 10;

// Vertical pitch of one digit, in em. Cells sit this far apart and the viewport
// is exactly this tall, so only one digit shows. Each glyph is centered (not
// baseline-aligned) so it has slack on both sides and never clips.
const STEP = 0.9;

const mod10 = (n: number) => ((n % 10) + 10) % 10;

// One rolling reel. `pos` is an unbounded position whose digit is `pos mod 10`;
// each cell is absolutely placed at its own index, so the reel can climb past 9
// into a repeated 0 (or below 0 into a 9) and always travels the short way.
function Reel({ digit }: { digit: number }) {
  const [pos, setPos] = useState(digit);

  useEffect(() => {
    setPos((prev) => {
      let delta = mod10(digit - prev); // 0..9, the forward distance
      if (delta > 5) delta -= 10; // prefer the shorter backward path
      return prev + delta;
    });
  }, [digit]);

  const cells = [];
  for (let i = pos - RADIUS; i <= pos + RADIUS; i++) cells.push(i);

  return (
    <span
      aria-hidden="true"
      // tracking-normal: the parent's negative letter-spacing would squeeze the
      // box narrower than the glyph and clip its sides. We restore the tight look
      // with a negative right margin between reels instead (mirrors the -5px the
      // readout sets on the whole number).
      className="relative -mr-1.25 inline-block overflow-hidden align-top tracking-normal tabular-nums"
      style={{ height: `${STEP}em` }}
    >
      {/* Sizes the reel to one digit; never painted. */}
      <span className="invisible leading-none">0</span>
      <span
        className="absolute inset-x-0 top-0 transition-transform duration-300 ease-out will-change-transform motion-reduce:transition-none"
        style={{ transform: `translateY(${-pos * STEP}em)` }}
      >
        {cells.map((i) => (
          <span
            className="absolute inset-x-0 flex items-center justify-center leading-none"
            key={i}
            style={{ top: `${i * STEP}em`, height: `${STEP}em` }}
          >
            {mod10(i)}
          </span>
        ))}
      </span>
    </span>
  );
}

// Renders a numeric string with an odometer roll on each digit. Digits are keyed
// by position from the right so the ones place stays put when the value crosses
// a digit boundary (99 -> 100), and non-digit glyphs (e.g. the "--" placeholder)
// render statically.
export function Odometer({
  value,
  className,
}: {
  value: string;
  className?: string;
}) {
  return (
    <span className={className}>
      <span className="sr-only">{value}</span>
      {value.split("").map((char, i) => {
        const place = value.length - 1 - i;
        const digit = Number.parseInt(char, 10);
        return Number.isNaN(digit) ? (
          <span aria-hidden="true" key={`c${place}`}>
            {char}
          </span>
        ) : (
          <Reel key={`d${place}`} digit={digit} />
        );
      })}
    </span>
  );
}
