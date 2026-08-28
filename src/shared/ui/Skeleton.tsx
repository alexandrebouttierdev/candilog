import { cn } from "@/shared/lib/cn";

/**
 * Bloc de chargement.
 *
 * Le guide impose des squelettes plutôt qu'un indicateur centré : la forme de l'écran reste
 * lisible pendant le chargement, et le contenu ne provoque pas de saut de mise en page quand
 * il arrive. L'animation ne touche que l'opacité, conformément à la règle transverse qui
 * interdit d'animer la mise en page.
 */
export function Skeleton({ className }: { className?: string }) {
  return (
    <div
      aria-hidden="true"
      className={cn("animate-pulse rounded-pill bg-neutral-tint", className)}
    />
  );
}

/** Rows de squelette d'un tableau, dimensionnées sur la hauteur de ligne du guide. */
export function SkeletonRows({ rows = 5, columns = 4 }: { rows?: number; columns?: number }) {
  return (
    <div role="status" aria-label="Chargement en cours">
      {Array.from({ length: rows }, (_, row) => (
        <div key={row} className="flex h-row items-center gap-4 border-b border-line px-4">
          {Array.from({ length: columns }, (_, column) => (
            <Skeleton
              key={column}
              className={cn("h-3", column === 0 ? "flex-[2]" : "flex-1")}
            />
          ))}
        </div>
      ))}
    </div>
  );
}
