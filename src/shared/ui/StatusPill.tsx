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

const TONES: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

/**
 * Pastille de statut : fond teinté, icône 14 px, libellé 11,5 px/550, rayon 6 px.
 *
 * Reprend exactement la pastille des tableaux et des listes des maquettes ; `compact`
 * donne la variante sans icône des compteurs (rayon 5 px, padding réduit).
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
        "inline-flex h-5 items-center gap-[7px] font-medium whitespace-nowrap",
        compact
          ? "rounded-chip px-1.5 text-label"
          : "rounded-pill px-2 text-label",
        TONES[tone],
        className,
      )}
    >
      {icon ? <Icon name={icon} size={14} /> : null}
      {children}
    </span>
  );
}

/** Pastille grise d'un attribut sans tonalité (type de contrat, technologie). */
export function Tag({ children, className }: { children: React.ReactNode; className?: string }) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-chip bg-neutral-tint px-[7px] py-[2px] text-tag font-mid text-ink-muted",
        className,
      )}
    >
      {children}
    </span>
  );
}
