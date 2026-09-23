import { cn } from "@/shared/lib/cn";
import { formatShortcut } from "@/shared/lib/platform";

/**
 * Pastille de touche clavier (`COMPONENTS.md` §7 du design) : mono 10 px, 17 px de haut,
 * dimensionnée par son contenu — `Ctrl ⏎` ne doit jamais être tronqué ni passer à la ligne.
 *
 * `shortcut` suit la notation neutre de `formatShortcut` (`mod+k`, `n`, `enter`) : la
 * pastille imprime ce qu'attend la plateforme courante.
 */
/**
 * Tons de pastille : `chip` (défaut), `accent` (« Actions ⌘K » de la barre d'état),
 * `on-accent` (posée sur un bouton primaire), `ghost` (menus et palette, sans fond).
 */
export type KbdTone = "chip" | "accent" | "on-accent" | "ghost";

const TONS: Record<KbdTone, string> = {
  chip: "bg-chip text-tx-2",
  accent: "bg-ac text-white",
  "on-accent": "bg-white/20 text-white",
  ghost: "bg-transparent text-tx-6",
};

export function Kbd({
  shortcut,
  tone = "chip",
  decorative = false,
  className,
}: {
  shortcut: string;
  tone?: KbdTone;
  /** Doublon visuel d'un raccourci déjà annoncé ailleurs (`aria-keyshortcuts`). */
  decorative?: boolean;
  /** Positionnement chez l'appelant (marges), jamais la couleur. */
  className?: string;
}) {
  const label = formatShortcut(shortcut);
  return (
    <kbd
      {...(decorative ? { "aria-hidden": true } : {})}
      className={cn(
        "inline-flex h-[17px] min-w-[17px] flex-none items-center justify-center rounded-r4 font-mono text-mono-sm whitespace-nowrap",
        label.length > 2 ? "px-[5px]" : "px-1",
        TONS[tone],
        className,
      )}
    >
      {label}
    </kbd>
  );
}
