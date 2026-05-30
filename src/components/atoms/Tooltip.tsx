import type { DOMAttributes, FocusableElement } from "@react-types/shared";
import type { ReactElement, ReactNode } from "react";
import {
  Focusable,
  Tooltip as RACTooltip,
  TooltipTrigger,
} from "react-aria-components";

type Placement = "top" | "bottom" | "left" | "right";

type FocusableChild = ReactElement<DOMAttributes<FocusableElement>, string>;

interface TooltipProps {
  content: ReactNode;
  children: ReactElement<DOMAttributes<FocusableElement>>;
  placement?: Placement;
  delay?: number;
  closeDelay?: number;
  isDisabled?: boolean;
}

export const Tooltip = ({
  content,
  children,
  placement = "top",
  delay = 300,
  closeDelay = 0,
  isDisabled,
}: TooltipProps) => (
  <TooltipTrigger delay={delay} closeDelay={closeDelay} isDisabled={isDisabled}>
    <Focusable>{children as FocusableChild}</Focusable>
    <RACTooltip
      placement={placement}
      offset={8}
      className="rounded-lg border border-line-strong bg-surface-2 px-3 py-2 text-xs font-medium text-fg shadow-lg"
    >
      {content}
    </RACTooltip>
  </TooltipTrigger>
);
