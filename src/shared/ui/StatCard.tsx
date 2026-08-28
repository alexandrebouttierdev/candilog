import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import type { Tone } from "./StatusPill";

/** Teinte de fond de la pastille d'icône, par tonalité. */
const TILE: Record<Tone, string> = {
  neutral: "bg-neutral-tint text-ink-muted",
  accent: "bg-accent-tint text-accent",
  success: "bg-success-tint text-success",
  warning: "bg-warning-tint text-warning",
  danger: "bg-danger-tint text-danger",
};

const DELTA: Record<Tone, string> = {
  neutral: "text-ink-faint",
  accent: "text-accent",
  success: "text-success",
  warning: "text-warning",
  danger: "text-danger",
};

/**
 * Indicateur chiffré du tableau de bord et des analyses.
 *
 * Géométrie des maquettes : carte de 16 px / 18 px, libellé 12 px à gauche et pastille
 * d'icône de 26 px à droite, puis la valeur en 26 px/650 alignée sur la ligne de base du
 * delta. La valeur est en chiffres tabulaires : sans cela, une rangée de KPI qui se
 * rafraîchit voit ses chiffres changer de largeur et danser d'un rendu à l'autre.
 */
export function StatCard({
  icon,
  tone = "accent",
  label,
  value,
  delta,
  deltaIcon,
  deltaTone = "neutral",
  className,
}: {
  icon: string;
  /** Tonalité de la pastille d'icône. */
  tone?: Tone;
  label: string;
  value: string;
  /** Variation par rapport à la période précédente, déjà formatée (« +12 % »). */
  delta?: ReactNode;
  deltaIcon?: string;
  deltaTone?: Tone;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "min-w-0 rounded-card border border-line bg-surface px-[18px] py-4 shadow-e1",
        className,
      )}
    >
      <div className="mb-[13px] flex items-center justify-between gap-2">
        <span className="min-w-0 truncate text-note font-medium text-ink-muted">{label}</span>
        <span
          className={cn(
            "flex size-[26px] flex-none items-center justify-center rounded-button",
            TILE[tone],
          )}
        >
          <Icon name={icon} size={16} />
        </span>
      </div>
      <div className="flex items-baseline gap-2">
        <span className="tabular text-kpi text-ink">{value}</span>
        {delta ? (
          <span
            className={cn(
              "inline-flex items-center gap-[3px] text-label font-mid",
              DELTA[deltaTone],
            )}
          >
            {deltaIcon ? <Icon name={deltaIcon} size={14} /> : null}
            {delta}
          </span>
        ) : null}
      </div>
    </div>
  );
}
