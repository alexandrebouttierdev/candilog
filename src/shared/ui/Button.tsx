import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";
import type { IconName } from "./icon-names";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

const VARIANTS: Record<ButtonVariant, string> = {
  primary:
    "border border-on-accent-border bg-accent font-semibold text-on-accent hover:bg-accent-hover",
  secondary:
    "border border-control-strong bg-fill font-semibold text-ink hover:bg-fill-hover",
  ghost: "font-semibold text-ink-muted hover:text-ink",
  danger: "border border-danger-border bg-danger-tint font-semibold text-danger-text hover:bg-danger-tint",
};

const SIZES = {
  control: { height: "h-control", icon: 16, pad: { large: "px-[13px]", small: "pl-[9px] pr-[11px]" } },
  dialog: { height: "h-[32px]", icon: 16, pad: { large: "px-[13px]", small: "px-[13px]" } },
} as const;

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: keyof typeof SIZES;
  icon?: IconName;
  children?: ReactNode;
}

export function Button({
  variant = "secondary",
  size = "control",
  icon,
  children,
  className,
  type = "button",
  ...props
}: ButtonProps) {
  const gabarit = SIZES[size];

  return (
    <button
      type={type}
      className={cn(
        "inline-flex items-center justify-center gap-1.5 rounded-button text-item whitespace-nowrap",
        "transition-[background-color,border-color,color] duration-hover ease-in-out",
        "focus-visible:outline-1 focus-visible:outline-accent-focus",
        "disabled:pointer-events-none disabled:border-transparent disabled:bg-fill disabled:text-ink-faint",
        gabarit.height,
        variant === "primary" ? gabarit.pad.large : gabarit.pad.small,
        VARIANTS[variant],
        className,
      )}
      {...props}
    >
      {icon ? <Icon name={icon} size={gabarit.icon} /> : null}
      {children}
    </button>
  );
}

export function IconButton({
  icon,
  label,
  size = 17,
  className,
  type = "button",
  ...props
}: Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  icon: IconName;
  label: string;
  size?: number;
}) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cn(
        "flex size-[30px] flex-none items-center justify-center rounded-button",
        "border border-control bg-fill text-ink-muted",
        "transition-colors duration-hover ease-in-out hover:bg-fill-hover hover:text-ink",
        "focus-visible:outline-1 focus-visible:outline-accent-focus",
        "disabled:pointer-events-none disabled:text-ink-faint/50",
        className,
      )}
      {...props}
    >
      <Icon name={icon} size={size} />
    </button>
  );
}
