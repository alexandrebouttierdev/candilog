import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import type { Tone } from "./StatusPill";

const DELTA: Record<Tone, string> = {
  neutral: "text-ink-faint",
  accent: "text-accent",
  success: "text-success",
  warning: "text-warning",
  danger: "text-danger",
};

/**
 * Metric chiffré du tableau de bord et des analyses.
 *
 * Libellé 12 px, puis la valeur en 26 px/650 alignée sur la ligne de base du delta. La
 * valeur est en chiffres tabulaires : sans cela, une rangée de KPI qui se rafraîchit voit
 * ses chiffres changer de largeur et danser d'un rendu à l'autre.
 */
export function StatCard({
  label,
  value,
  delta,
  deltaTone = "neutral",
  className,
}: {
  label: string;
  value: string;
  /** Variation par rapport à la période précédente, déjà formatée (« +12 % »). */
  delta?: ReactNode;
  deltaTone?: Tone;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "min-w-0 rounded-card border border-line bg-surface px-[11px] py-2.5",
        className,
      )}
    >
      <p className="mb-[13px] truncate text-note font-medium text-ink-muted">{label}</p>
      <div className="flex items-baseline gap-2">
        <span className="tabular text-kpi text-ink">{value}</span>
        {delta ? (
          <span
            className={cn(
              "inline-flex items-center gap-[3px] text-label font-mid",
              DELTA[deltaTone],
            )}
          >
            {delta}
          </span>
        ) : null}
      </div>
    </div>
  );
}
