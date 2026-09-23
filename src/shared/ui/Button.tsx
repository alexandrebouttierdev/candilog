import type { ButtonHTMLAttributes, ComponentPropsWithRef, ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";
import { Kbd } from "./Kbd";
import type { IconName } from "./icon-names";

/**
 * Variantes du design (`COMPONENTS.md` §1) :
 * - `primary` : accent plein, une seule par écran ou par modale ;
 * - `secondary` : `bg-chip`, actions de même rang que la primaire ;
 * - `ghost` (« discret ») : transparent, encre `tx-5` montant à `tx-3` au survol ;
 * - `danger` (« destructeur ») : contour `st-c` sur fond transparent — jamais de rouge
 *   plein, et uniquement pour confirmer une destruction ;
 * - `link` : lien d'action en `ac-tx` (« changer », « annuler »).
 */
export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "link";

const VARIANTS: Record<ButtonVariant, string> = {
  primary: "bg-ac px-3 font-medium text-white hover:brightness-110",
  secondary: "bg-chip px-[11px] font-medium text-tx-2 hover:bg-elev",
  ghost: "bg-transparent px-2.5 text-tx-5 hover:text-tx-3",
  danger: "border border-st-c bg-transparent px-[11px] font-medium text-st-c hover:bg-tint-c-bg",
  link: "h-auto bg-transparent px-0 text-ac-tx hover:underline",
};

/**
 * Hauteurs du design : 27 px par défaut, 26 px dans une barre, 23 px en barre d'outils
 * compacte, 28 px dans un état vide. `control` et `dialog` sont conservés pour les écrans
 * qui n'ont pas encore migré : tous deux valent désormais la hauteur par défaut.
 */
const SIZES = {
  control: "h-btn",
  dialog: "h-btn",
  bar: "h-[26px]",
  compact: "h-[23px]",
  empty: "h-7",
} as const;

interface ButtonProps extends ComponentPropsWithRef<"button"> {
  variant?: ButtonVariant;
  size?: keyof typeof SIZES;
  icon?: IconName;
  /** Raccourci imprimé à droite du libellé (`mod+enter`, `n`…), notation de la plateforme. */
  shortcut?: string;
  children?: ReactNode;
}

export function Button({
  variant = "secondary",
  size = "control",
  icon,
  shortcut,
  children,
  className,
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button
      type={type}
      data-variant={variant}
      {...(shortcut ? { "aria-keyshortcuts": ariaShortcut(shortcut) } : {})}
      className={cn(
        // `nowrap` + `flex-none` : un libellé long ou « Ctrl ⏎ » ne passe jamais sur deux
        // lignes et ne comprime pas son voisin (`COMPONENTS.md` §10).
        "inline-flex flex-none items-center justify-center gap-1.5 rounded-r7 text-ui whitespace-nowrap",
        "transition-color",
        // Désactivé : fond `bg-chip`, encre `tx-5`, hors de la tabulation (le navigateur
        // retire déjà un bouton `disabled` de l'ordre de focus).
        "disabled:pointer-events-none disabled:border-transparent disabled:bg-chip disabled:text-tx-5 disabled:brightness-100",
        variant === "link" ? "" : SIZES[size],
        VARIANTS[variant],
        className,
      )}
      {...props}
    >
      {icon ? (
        <Icon
          name={icon}
          size={15}
          {...(icon === "progress_activity" ? { className: "animate-spin" } : {})}
        />
      ) : null}
      {children}
      {shortcut ? (
        // Décorative : le raccourci est annoncé par `aria-keyshortcuts`, pas dans le nom du
        // bouton (« Supprimer ⏎ » n'est pas un nom).
        <Kbd shortcut={shortcut} tone={variant === "primary" ? "on-accent" : "chip"} decorative className="-mr-1 ml-0.5" />
      ) : null}
    </button>
  );
}

/** Notation ARIA d'un raccourci neutre : `mod+enter` → `Meta+Enter Control+Enter`. */
function ariaShortcut(shortcut: string): string {
  const touches = shortcut.split("+").map((part) => {
    const lower = part.toLowerCase();
    if (lower === "enter") return "Enter";
    if (lower === "shift") return "Shift";
    if (lower === "alt") return "Alt";
    if (lower === "backspace") return "Backspace";
    if (lower === "escape") return "Escape";
    return part.length === 1 ? part.toUpperCase() : part;
  });
  if (touches[0]?.toLowerCase() !== "mod") return touches.join("+");
  const reste = touches.slice(1).join("+");
  return `Meta+${reste} Control+${reste}`;
}

/**
 * Bouton carré d'icône (`✕`, `⋯`, `+`) : 24 px par défaut, fond `bg-chip`, rayon 7.
 *
 * Le libellé est obligatoire : c'est le nom accessible et l'infobulle d'un contrôle sans
 * texte.
 */
export function IconButton({
  icon,
  label,
  size = 15,
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
        "flex size-6 flex-none items-center justify-center rounded-r7",
        "bg-chip text-tx-4 transition-color hover:bg-elev hover:text-tx-2",
        "disabled:pointer-events-none disabled:text-tx-6",
        className,
      )}
      {...props}
    >
      <Icon name={icon} size={size} />
    </button>
  );
}

/**
 * Bouton carré à glyphe typographique (`✕`, `⋯`, `+`, `✎`) — les micro-icônes que le design
 * garde en texte (`DESIGN_SYSTEM.md` §7). Même gabarit qu'`IconButton`, sans police d'icônes.
 */
export function GlyphButton({
  glyph,
  label,
  className,
  type = "button",
  ...props
}: Omit<ComponentPropsWithRef<"button">, "children"> & {
  glyph: string;
  label: string;
}) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cn(
        "flex size-[22px] flex-none items-center justify-center rounded-r6",
        "bg-chip text-small leading-none text-tx-4 transition-color hover:bg-elev hover:text-tx-2",
        "disabled:pointer-events-none disabled:text-tx-6",
        className,
      )}
      {...props}
    >
      <span aria-hidden>{glyph}</span>
    </button>
  );
}
