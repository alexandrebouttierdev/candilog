/**
 * Jetons de couleur des graphiques.
 *
 * Les valeurs sont des `var(--candilog-…)` et non des hexadécimaux : le SVG résout la
 * variable au rendu, si bien qu'un basculement clair / sombre repeint les graphiques sans
 * remonter dans React. C'est aussi ce qui interdit à Recharts d'imposer sa propre palette.
 */
export const CHART_INK = {
  /** Série unique et étapes en progression. */
  accent: "var(--candilog-accent)",
  success: "var(--candilog-success)",
  warning: "var(--candilog-warning)",
  danger: "var(--candilog-danger)",
  /** Étape neutre du pipeline : un gris de texte, jamais une teinte porteuse de sens. */
  neutral: "var(--candilog-text-disabled)",
  /** Fond des barres à zéro, pour que la catégorie reste visible. */
  empty: "var(--candilog-neutral-tint)",
  grid: "var(--candilog-border)",
  axis: "var(--candilog-text-label)",
  /** Filet de séparation entre deux segments contigus. */
  gap: "var(--candilog-surface)",
} as const;

/** Typographie des axes, alignée sur `text-eyebrow` du design system. */
export const AXIS_TICK = {
  fill: CHART_INK.axis,
  fontSize: 10.5,
} as const;

/**
 * Aire de tracé sans gouttière superflue.
 *
 * Recharts réserve par défaut de larges marges autour du tracé ; les cartes de Candilog
 * sont denses et fournissent déjà leur propre respiration.
 */
export const PLOT_MARGIN = { top: 8, right: 4, bottom: 0, left: 4 } as const;
