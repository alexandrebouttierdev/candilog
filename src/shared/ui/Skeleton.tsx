import { cn } from "@/shared/lib/cn";

/**
 * Bloc de chargement : fond `sk`, animation `sk` (opacité .45 ↔ .9 en 1 400 ms).
 *
 * Le design impose un squelette **de la forme réelle du contenu** plutôt qu'un indicateur
 * centré : l'écran reste lisible pendant le chargement et rien ne saute à l'arrivée des
 * données. `index` décale l'animation de 100 ms par ligne.
 */
export function Skeleton({ className, index = 0 }: { className?: string; index?: number }) {
  return (
    <div
      aria-hidden="true"
      style={index ? { animationDelay: `${index * 100}ms` } : undefined}
      className={cn("animate-sk rounded-r3 bg-sk", className)}
    />
  );
}

/** Lignes de squelette d'un tableau, dimensionnées sur la hauteur de ligne du guide. */
export function SkeletonRows({ rows = 5, columns = 4 }: { rows?: number; columns?: number }) {
  return (
    <div role="status" aria-label="Chargement en cours">
      {Array.from({ length: rows }, (_, row) => (
        <div key={row} className="flex h-row-app items-center gap-[11px] px-3.5">
          {Array.from({ length: columns }, (_, column) => (
            <Skeleton
              key={column}
              index={row}
              className={cn("h-[9px]", column === 0 ? "flex-[2]" : "flex-1")}
            />
          ))}
        </div>
      ))}
    </div>
  );
}
