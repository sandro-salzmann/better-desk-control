import {
  composeRenderProps,
  Button as RACButton,
  type ButtonProps as RACButtonProps,
} from "react-aria-components";

type Variant = "primary" | "secondary" | "ghost";
type Size = "sm" | "md" | "lg";
export type ButtonTone = "neutral" | "accent" | "lower" | "bluetooth" | "stop";

interface ButtonProps extends RACButtonProps {
  variant?: Variant;
  tone?: ButtonTone;
  size?: Size;
  /** Fixed square footprint for an icon-only button. */
  square?: boolean;
  fullWidth?: boolean;
}

// Resting · hover · pressed classes per variant × tone.
const variantToneClasses: Record<Variant, Record<ButtonTone, string>> = {
  primary: {
    neutral:
      "border border-line-strong bg-surface-1 text-fg hover:bg-surface-2 pressed:bg-surface-3",
    accent:
      "bg-accent text-surface-0 hover:brightness-105 pressed:brightness-95",
    lower: "bg-lower text-surface-0 hover:brightness-105 pressed:brightness-95",
    bluetooth:
      "bg-bluetooth text-surface-0 hover:brightness-105 pressed:brightness-95",
    stop: "bg-stop text-surface-0 hover:brightness-105 pressed:brightness-95",
  },
  secondary: {
    neutral:
      "border border-line-strong bg-surface-1 text-fg hover:border-line-hover hover:bg-surface-2 pressed:bg-surface-3",
    accent:
      "border border-accent/35 bg-accent/10 text-accent hover:border-accent/55 hover:bg-accent/15 pressed:bg-accent/20",
    lower:
      "border border-lower/40 bg-lower/10 text-lower hover:border-lower/55 hover:bg-lower/15 pressed:bg-lower/20",
    bluetooth:
      "border border-bluetooth/40 bg-bluetooth/10 text-bluetooth hover:border-bluetooth/55 hover:bg-bluetooth/15 pressed:bg-bluetooth/20",
    stop: "border border-stop/40 bg-stop/10 text-stop hover:border-stop/55 hover:bg-stop/15 pressed:bg-stop/20",
  },
  ghost: {
    neutral:
      "text-fg-muted hover:bg-surface-1 hover:text-fg pressed:bg-surface-2",
    accent: "text-accent hover:bg-accent/10 pressed:bg-accent/15",
    lower: "text-lower hover:bg-lower/10 pressed:bg-lower/15",
    bluetooth: "text-bluetooth hover:bg-bluetooth/10 pressed:bg-bluetooth/15",
    stop: "text-stop hover:bg-stop/10 pressed:bg-stop/15",
  },
};

const sizeClasses: Record<Size, string> = {
  sm: "px-3 py-1 text-xs",
  md: "px-4 py-3 text-sm",
  lg: "px-6 py-3 text-base",
};

// Icon-only squares: fixed footprint + matching glyph size.
const squareSizeClasses: Record<Size, string> = {
  sm: "h-8 w-8 [&_svg]:h-4 [&_svg]:w-4",
  md: "h-10 w-10 [&_svg]:h-5 [&_svg]:w-5",
  lg: "h-11 w-11 [&_svg]:h-5 [&_svg]:w-5",
};

export const Button = ({
  variant = "secondary",
  tone = "neutral",
  size = "md",
  square = false,
  fullWidth = false,
  children,
  ...props
}: ButtonProps) => {
  const stateClasses = variantToneClasses[variant][tone];
  const padClasses = square ? squareSizeClasses[size] : sizeClasses[size];
  return (
    <RACButton
      {...props}
      className={composeRenderProps(
        props.className,
        (className, { isDisabled, isFocusVisible }) =>
          `inline-flex items-center justify-center gap-2 rounded-xl font-semibold transition outline-none
          ${stateClasses} ${padClasses} ${fullWidth ? "w-full" : ""}
          ${isFocusVisible ? "ring-2 ring-accent/50" : ""}
          ${isDisabled ? "opacity-40 pointer-events-none" : "cursor-pointer"}
          ${className ?? ""}`,
      )}
    >
      {children}
    </RACButton>
  );
};
