import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";
import type { IconName } from "./icon-names";

/**
 * Tonalité sémantique d'un statut.
 *
 * Le guide impose que la couleur réponde toujours à une question : vert = avancement,
 * ambre = à traiter, rouge = échec, neutre = en attente. Elle ne porte jamais l'information
 * seule — le libellé l'accompagne systématiquement, condition du critère de contraste.
 */
export type Tone = "neutral" | "accent" | "success" | "warning" | "danger";

/**
 * Paires teinte / encre du design (`DESIGN_SYSTEM.md` §1.6), indissociables : une encre
 * posée sur une autre teinte perd son contraste. L'ambre n'a pas de paire dans le design ;
 * il reprend la couleur d'attention sur un fond qui en dérive.
 */
const TONES: Record<Tone, string> = {
  neutral: "bg-chip text-tx-3",
  accent: "bg-tint-ac-bg text-tint-ac-tx",
  success: "bg-tint-g-bg text-tint-g-tx",
  warning: "bg-warning-tint text-st-a",
  danger: "bg-tint-c-bg text-tint-c-tx",
};

/**
 * Pastille (`COMPONENTS.md` §7) : 20 px, rayon 5, retrait `0 7px`, texte 12 px.
 *
 * `compact` donne la variante 19 px à retrait `0 6px` des listes denses.
 */
export function StatusPill({
  tone = "neutral",
  icon,
  compact = false,
  children,
  className,
}: {
  tone?: Tone;
  icon?: IconName;
  compact?: boolean;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-r5 text-small whitespace-nowrap",
        compact ? "h-[19px] px-1.5" : "h-5 px-[7px]",
        TONES[tone],
        className,
      )}
    >
      {icon ? <Icon name={icon} size={13} /> : null}
      {children}
    </span>
  );
}

/** Pastille neutre d'un attribut sans tonalité (type de contrat, technologie). */
export function Tag({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <span
      className={cn(
        "inline-flex h-5 items-center rounded-r5 bg-chip px-[7px] text-small whitespace-nowrap text-tx-3",
        className,
      )}
    >
      {children}
    </span>
  );
}
