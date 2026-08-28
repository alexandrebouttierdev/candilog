import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import type { Tone } from "./StatusPill";

/**
 * Indicateur chiffré du tableau de bord et des analyses.
 *
 * La valeur est en chiffres tabulaires : sans cela, une rangée de KPI qui se rafraîchit
 * voit ses chiffres changer de largeur et danser d'un rendu à l'autre.
 */
export function StatCard({
  icon,
  label,
  value,
  delta,
  deltaTone = "neutral",
  className,
}: {
  icon: string;
  label: string;
  value: string;
  /** Variation par rapport à la période précédente, déjà formatée (« +12 % »). */
  delta?: string;
  deltaTone?: Tone;
  className?: string;
}) {
  const deltaColor: Record<Tone, string> = {
    neutral: "text-ink-faint",
    accent: "text-accent",
    success: "text-success",
    warning: "text-warning",
    danger: "text-danger",
  };

  return (
    <div className={cn("rounded-card border border-line bg-surface p-4 shadow-e1", className)}>
      <div className="flex items-center gap-2">
        <span className="flex size-7 items-center justify-center rounded-pill bg-accent-tint text-accent">
          <Icon name={icon} size={15} />
        </span>
        <p className="min-w-0 flex-1 truncate text-meta text-ink-muted">{label}</p>
      </div>
      <p className="tabular mt-2 text-kpi text-ink">{value}</p>
      {delta ? <p className={cn("tabular text-meta", deltaColor[deltaTone])}>{delta}</p> : null}
    </div>
  );
}
