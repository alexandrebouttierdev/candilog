import { cn } from "@/shared/lib/cn";
import { Icon } from "./Icon";

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

export function StatusPill({
  tone = "neutral",
  icon,
  children,
  className,
}: {
  tone?: Tone;
  icon?: string;
  children: string;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-pill px-2 py-[3px] text-meta font-medium",
        TONES[tone],
        className,
      )}
    >
      {icon ? <Icon name={icon} size={13} /> : null}
      {children}
    </span>
  );
}
