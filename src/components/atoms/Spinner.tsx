type Size = "xs" | "sm" | "lg";
type Tone = "accent" | "bluetooth";

interface SpinnerProps {
  size?: Size;
  tone?: Tone;
  className?: string;
}

const sizeClasses: Record<Size, string> = {
  xs: "h-3 w-3 border-[1.5px]",
  sm: "h-4 w-4 border-2",
  lg: "h-11 w-11 border-[3px]",
};

const toneClasses: Record<Tone, string> = {
  accent: "border-t-accent",
  bluetooth: "border-t-bluetooth",
};

export const Spinner = ({
  size = "lg",
  tone = "accent",
  className = "",
}: SpinnerProps) => (
  <div
    className={`shrink-0 animate-spin rounded-full border-line-strong ${sizeClasses[size]} ${toneClasses[tone]} ${className}`}
  />
);
