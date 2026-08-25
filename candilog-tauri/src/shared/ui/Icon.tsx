/**
 * Icône Material Symbols Rounded, jeu retenu par les maquettes SPECDESIGN.
 *
 * La police est embarquée localement (`shared/ui/material-symbols.css`) : la politique de
 * sécurité de contenu de la fenêtre Tauri interdit les requêtes vers des hôtes externes,
 * et une application de bureau doit rester utilisable hors ligne.
 */
export function Icon({
  name,
  size = 18,
  className = "",
  filled = false,
}: {
  name: string;
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
        fontVariationSettings: `'FILL' ${filled ? 1 : 0}, 'wght' 400, 'GRAD' 0, 'opsz' ${size}`,
      }}
    >
      {name}
    </span>
  );
}
