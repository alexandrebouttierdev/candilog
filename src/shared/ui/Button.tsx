import type { ButtonHTMLAttributes, ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";

/**
 * Variantes de bouton du guide SPECDESIGN.
 *
 * `primary` est **unique par écran** : le guide n'admet qu'une seule action primaire, le
 * reste passe en `secondary` (outline) ou en `ghost`. `danger` est réservé aux actions
 * destructives, qui doivent en outre être isolées et confirmées.
 */
export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

const VARIANTS: Record<ButtonVariant, string> = {
  primary: "bg-accent text-white shadow-e1 hover:brightness-110",
  secondary: "border border-line bg-surface text-ink hover:bg-neutral-tint",
  ghost: "text-ink-muted hover:bg-neutral-tint hover:text-ink",
  danger: "border border-danger/40 bg-transparent text-danger hover:bg-danger-tint",
};

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  /** Icône Material Symbols placée avant le libellé. */
  icon?: string;
  children?: ReactNode;
}

export function Button({
  variant = "secondary",
  icon,
  children,
  className,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cn(
        // Hauteur 33 px et rayon 8 px imposés par le guide ; la cible reste au-dessus du
        // minimum de 32 px exigé pour l'accessibilité.
        "inline-flex h-control items-center justify-center gap-1.5 rounded-button px-3.5",
        "text-body font-medium whitespace-nowrap",
        "transition-[background-color,border-color,color,filter] duration-150",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        "disabled:pointer-events-none disabled:opacity-45",
        VARIANTS[variant],
        className,
      )}
      {...props}
    >
      {icon ? <Icon name={icon} size={16} /> : null}
      {children}
    </button>
  );
}
