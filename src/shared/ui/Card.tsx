import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";

/**
 * Surface de contenu des maquettes : filet 1 px, rayon 12 px, ombre de niveau 1.
 *
 * `padded` applique le padding interne des cartes de contenu (17 px / 19 px) ; on
 * l'omet pour les cartes qui portent un en-tête à filet et des lignes pleine largeur,
 * lesquelles doivent aussi passer `clipped` pour que le rayon rogne la première ligne.
 */
export function Card({
  padded = false,
  clipped = false,
  className,
  children,
}: {
  padded?: boolean;
  clipped?: boolean;
  className?: string;
  children: ReactNode;
}) {
  return (
    <div
      className={cn(
        "min-w-0 rounded-card border border-line bg-surface",
        padded && "px-[19px] py-[17px]",
        clipped && "overflow-hidden",
        className,
      )}
    >
      {children}
    </div>
  );
}

/**
 * Title de section interne à une carte sans filet : libellé 13,5 px/600, méta optionnelle
 * poussée à droite.
 */
export function CardTitle({
  children,
  meta,
  compact = false,
  className,
}: {
  children: ReactNode;
  meta?: ReactNode;
  /** Title 12,5 px des fiches Relations, au lieu des 13,5 px des cartes de tableau de bord. */
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("flex items-center justify-between gap-3", className)}>
      <span className={cn("truncate text-ink", compact ? "text-body font-semibold" : "text-section")}>
        {children}
      </span>
      {meta}
    </div>
  );
}

/**
 * En-tête à filet d'une carte-tableau : même titre, bande de 14 px / 19 px — 13 px / 17 px
 * dans sa variante compacte, celle des fiches Relations.
 */
export function CardHeader({
  children,
  meta,
  compact = false,
}: {
  children: ReactNode;
  meta?: ReactNode;
  compact?: boolean;
}) {
  return (
    <CardTitle
      {...(meta ? { meta } : {})}
      compact={compact}
      className={cn(
        "border-b border-line",
        compact ? "px-[17px] py-[13px]" : "px-[19px] py-[14px]",
      )}
    >
      {children}
    </CardTitle>
  );
}

/** Métadonnée grise à droite d'un titre de carte (« 3 à venir », « 7 derniers jours »). */
export function CardMeta({ children }: { children: ReactNode }) {
  return <span className="flex-none text-label text-ink-faint">{children}</span>;
}
