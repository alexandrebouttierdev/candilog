import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/**
 * Bandeau de filtres sous l'en-tête de page.
 *
 * Géométrie des maquettes du Suivi : bande de 11 px / 28 px posée sur le fond de page — et
 * non sur la surface, contrairement à l'en-tête — avec les puces à gauche et le décompte
 * du résultat à droite.
 */
export function FilterBar({ children, summary }: { children: ReactNode; summary?: ReactNode }) {
  return (
    <div className="flex flex-none flex-wrap items-center gap-2 border-b border-line bg-page px-7 py-[11px]">
      {children}
      <span className="flex-1" />
      {summary ? <span className="text-note text-ink-faint">{summary}</span> : null}
    </div>
  );
}

/**
 * Puce de filtre : icône, libellé, chevron.
 *
 * La puce affiche la **valeur retenue** quand le critère est actif, pas son intitulé : la
 * maquette montre « 30 derniers jours » et non « Période », de sorte que l'état du filtre se
 * lise sans ouvrir le panneau. Le chevron indique qu'elle ouvre un choix.
 */
export function FilterChip({
  icon,
  label,
  active = false,
  onClick,
}: {
  icon: string;
  label: string;
  active?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-pressed={active}
      className={cn(
        "inline-flex h-chip items-center gap-1.5 rounded-control border px-[11px]",
        "text-note font-medium transition-colors duration-150",
        active
          ? "border-accent-border bg-accent-tint text-accent"
          : "border-line bg-surface text-ink-muted hover:bg-neutral-tint",
      )}
    >
      <Icon name={icon} size={15} />
      {label}
      <Icon name="expand_more" size={15} className={active ? "text-accent" : "text-ink-faint"} />
    </button>
  );
}
