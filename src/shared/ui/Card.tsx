import type { ReactNode } from "react";
import { Icon } from "./Icon";
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
        "min-w-0 rounded-card border border-line bg-surface shadow-e1",
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
 * Titre de section interne à une carte sans filet : icône 17 px tertiaire, libellé
 * 13,5 px/600, méta optionnelle poussée à droite.
 */
export function CardTitle({
  icon,
  iconClassName,
  children,
  meta,
  compact = false,
  className,
}: {
  icon?: string;
  /** Teinte de l'icône lorsqu'elle porte un sens (ambre pour les relances). */
  iconClassName?: string;
  children: ReactNode;
  meta?: ReactNode;
  /** Titre 12,5 px des fiches Relations, au lieu des 13,5 px des cartes de tableau de bord. */
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("flex items-center justify-between gap-3", className)}>
      <div className="flex min-w-0 items-center gap-2">
        {icon ? (
          <Icon
            name={icon}
            size={17}
            className={cn("flex-none", iconClassName ?? "text-ink-faint")}
          />
        ) : null}
        <span className={cn("truncate text-ink", compact ? "text-body font-semibold" : "text-section")}>
          {children}
        </span>
      </div>
      {meta}
    </div>
  );
}

/**
 * En-tête à filet d'une carte-tableau : même titre, bande de 14 px / 19 px — 13 px / 17 px
 * dans sa variante compacte, celle des fiches Relations.
 */
export function CardHeader({
  icon,
  iconClassName,
  children,
  meta,
  compact = false,
}: {
  icon?: string;
  iconClassName?: string;
  children: ReactNode;
  meta?: ReactNode;
  compact?: boolean;
}) {
  return (
    <CardTitle
      {...(icon ? { icon } : {})}
      {...(iconClassName ? { iconClassName } : {})}
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

/**
 * Lien discret « Tout voir » des en-têtes de carte.
 *
 * `compact` donne la variante des fiches Relations : 11,5 px et pas de chevron, là où les
 * cartes du tableau de bord affichent le lien en 12,5 px suivi d'un chevron.
 */
export function CardLink({
  onClick,
  compact = false,
  children,
}: {
  onClick: () => void;
  compact?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "inline-flex flex-none items-center gap-1 rounded-pill font-medium text-accent transition-opacity duration-150 hover:opacity-80",
        compact ? "text-label" : "text-body",
      )}
    >
      {children}
      {compact ? null : <Icon name="chevron_right" size={16} />}
    </button>
  );
}
