import type { IconName } from "./icon-names";

/**
 * Icône Material Symbols Rounded, jeu retenu par le design system (`docs/DESIGN.md` §6).
 *
 * La police est embarquée localement (`shared/ui/material-symbols.css`), réduite aux seules
 * icônes de `icon-names.ts` : la politique de sécurité de contenu de la fenêtre Tauri
 * interdit les requêtes vers des hôtes externes, et une application de bureau doit rester
 * utilisable hors ligne.
 *
 * `name` est typé sur cette liste et non sur `string` : une icône absente de la sous-police
 * serait rendue en toutes lettres à l'écran, sans que rien ne le signale au développeur.
 */
export function Icon({
  name,
  size = 18,
  className = "",
  filled = false,
}: {
  name: IconName;
  size?: number;
  className?: string;
  filled?: boolean;
}) {
  return (
    <span
      aria-hidden="true"
      className={`material-symbols-rounded select-none leading-none ${className}`}
      style={{
        fontSize: size,
        fontVariationSettings: `'FILL' ${filled ? 1 : 0}, 'wght' 300, 'GRAD' 0, 'opsz' ${Math.min(size, 20)}`,
      }}
    >
      {name}
    </span>
  );
}
