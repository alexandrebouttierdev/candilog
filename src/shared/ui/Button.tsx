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

/**
 * Les maquettes distinguent le primaire du secondaire par la graisse autant que par la
 * couleur : `font:550` sur le premier, `font:500` sur le second, et un pas de padding de
 * plus pour compenser l'absence de filet. Le destructif ne porte pas de filet non plus,
 * mais un fond teinté rouge.
 */
const VARIANTS: Record<ButtonVariant, string> = {
  primary: "bg-accent font-mid text-white shadow-accent hover:brightness-110",
  secondary: "border border-line bg-surface font-medium text-ink shadow-e1 hover:bg-neutral-tint",
  ghost: "font-medium text-ink-muted hover:bg-neutral-tint hover:text-ink",
  danger: "bg-danger-tint font-mid text-danger hover:brightness-95",
};

/** Couleur de l'icône du secondaire : gris moyen dans les maquettes, pas la couleur du texte. */
const ICON_TONE: Record<ButtonVariant, string> = {
  primary: "",
  secondary: "text-ink-muted",
  ghost: "",
  danger: "",
};

/**
 * Les pieds de modale montent les boutons d'un pixel et élargissent le primaire d'autant :
 * seule différence de gabarit du guide, elle suffit à décaler une modale entière.
 */
const SIZES = {
  control: { height: "h-control", icon: 17, pad: { large: "px-[14px]", small: "px-[13px]" } },
  dialog: { height: "h-[34px]", icon: 16, pad: { large: "px-[15px]", small: "px-[14px]" } },
} as const;

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  /** Gabarit : hauteur de contrôle courante (33 px) ou de pied de modale (34 px). */
  size?: keyof typeof SIZES;
  /** Icône Material Symbols placée avant le libellé. */
  icon?: string;
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
        // Hauteur 33 px et rayon 8 px imposés par le guide ; la cible reste au-dessus du
        // minimum de 32 px exigé pour l'accessibilité.
        "inline-flex items-center justify-center gap-[7px] rounded-button",
        "text-body whitespace-nowrap",
        "transition-[background-color,border-color,color,filter] duration-150",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        "disabled:pointer-events-none disabled:border-transparent disabled:bg-neutral-tint",
        "disabled:text-ink-faint disabled:shadow-none",
        gabarit.height,
        variant === "primary" ? gabarit.pad.large : gabarit.pad.small,
        VARIANTS[variant],
        className,
      )}
      {...props}
    >
      {icon ? <Icon name={icon} size={gabarit.icon} className={ICON_TONE[variant]} /> : null}
      {children}
    </button>
  );
}

/**
 * Bouton sans libellé : fermeture d'une modale ou d'un panneau, action discrète d'en-tête.
 *
 * Cible de 28 px, icône 20 px, teinte grise qui s'assombrit au survol — la géométrie des
 * croix de fermeture des maquettes. Le `label` est obligatoire : sans texte visible, c'est
 * la seule chose qu'un lecteur d'écran peut annoncer.
 */
export function IconButton({
  icon,
  label,
  size = 20,
  className,
  type = "button",
  ...props
}: Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  icon: string;
  label: string;
  size?: number;
}) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cn(
        "flex size-7 flex-none items-center justify-center rounded-button text-ink-faint",
        "transition-colors duration-150 hover:bg-neutral-tint hover:text-ink",
        "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
        "disabled:pointer-events-none disabled:text-ink-faint/50",
        className,
      )}
      {...props}
    >
      <Icon name={icon} size={size} />
    </button>
  );
}
