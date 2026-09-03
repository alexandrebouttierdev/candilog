import type { ReactNode } from "react";

/**
 * Infobulle commune aux graphiques.
 *
 * Reprend la surface, le filet et l'ombre des popovers du design system plutôt que le style
 * par défaut de Recharts, qui est codé en dur en clair et devient illisible en thème sombre.
 */
export function ChartTooltip({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="rounded-field border border-line bg-surface px-2.5 py-1.5 shadow-e2">
      <p className="text-eyebrow uppercase tracking-[0.06em] text-ink-label">{title}</p>
      <p className="mt-0.5 text-note font-semibold text-ink">{children}</p>
    </div>
  );
}
