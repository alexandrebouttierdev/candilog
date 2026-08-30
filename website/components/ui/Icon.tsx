import { cn } from "@/lib/cn";

/**
 * Icône Material Symbols Rounded — même réglage que l'application desktop.
 *
 * La police est embarquée localement (paquet npm `material-symbols`, importé dans
 * `app/globals.css`) : aucune requête vers un hôte tiers au chargement.
 */
export function Icon({
  name,
  size = 18,
  className,
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
      className={cn("material-symbols-rounded block select-none leading-none", className)}
      style={{
        fontSize: size,
        fontVariationSettings: `'FILL' ${filled ? 1 : 0}, 'wght' 300, 'GRAD' 0, 'opsz' ${Math.min(size, 20)}`,
      }}
    >
      {name}
    </span>
  );
}
