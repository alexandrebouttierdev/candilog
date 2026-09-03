import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";
import type { IconName } from "./icon-names";

/** Bounds d'une page, calculées une fois pour l'affichage et pour les tests. */
export function page_bounds(page: number, page_size: number, total: number) {
  const page_count = Math.max(1, Math.ceil(total / page_size));
  const from = total === 0 ? 0 : (page - 1) * page_size + 1;
  const to = Math.min(page * page_size, total);
  return { page_count, from, to, hasPrev: page > 1, hasNext: page < page_count };
}

/**
 * Pagination réutilisable.
 *
 * Pied de 11 px / 19 px sur fond `surface-alt`, résumé à gauche, sélecteur de densité puis
 * boutons de 28 px à droite : la barre des maquettes du Tracking et des Relations.
 *
 * La pagination est **côté données** : le composant ne reçoit jamais la collection
 * complète, seulement la page courante et le total, et le ViewModel ne demande au backend
 * que la page affichée. Le guide exige de ne matérialiser qu'une page au-delà de 50
 * éléments ; passer la liste entière ici l'aurait rendu impossible par construction.
 */
export function Pager({
  page,
  page_size,
  total,
  label,
  pageSizes,
  dense = false,
  onPageChange,
  onPageSizeChange,
}: {
  page: number;
  page_size: number;
  total: number;
  /** Name de ce qui est compté, au pluriel : « candidatures », « entreprises ». */
  label: string;
  /** Densités proposées ; le sélecteur est masqué si l'appelant n'en gère pas. */
  pageSizes?: readonly number[];
  /** Pied resserré des colonnes maîtresses : 10 px / 14 px au lieu de 11 px / 19 px. */
  dense?: boolean;
  onPageChange: (page: number) => void;
  onPageSizeChange?: (page_size: number) => void;
}) {
  const { page_count, from, to, hasPrev, hasNext } = page_bounds(page, page_size, total);

  // Les trois premiers numéros puis une ellipse : le guide affiche un repère de position,
  // pas un index complet — au-delà, le compteur textuel est plus lisible qu'une rangée
  // de numéros.
  const numbers = Array.from({ length: Math.min(page_count, 3) }, (_, index) => index + 1);

  return (
    <nav
      aria-label={`Pagination des ${label}`}
      className={cn(
        "flex flex-none flex-wrap items-center border-t border-line bg-surface-alt",
        dense ? "gap-2.5 px-3.5 py-2.5" : "gap-3.5 px-[19px] py-[11px]",
      )}
    >
      <p
        className={cn(
          "tabular flex-1 truncate text-label text-ink-faint",
          dense ? "min-w-0" : "min-w-[150px]",
        )}
      >
        {from}–{to} sur {total} {label}
      </p>

      {pageSizes && onPageSizeChange ? (
        <div className="flex items-center gap-2">
          <span className="text-label text-ink-faint">Lignes</span>
          <div className="relative">
            <select
              value={page_size}
              aria-label="Nombre de lignes par page"
              onChange={(event) => onPageSizeChange(Number(event.target.value))}
              className="h-pager appearance-none rounded-control border border-line bg-surface pr-7 pl-2.5 text-note font-medium text-ink"
            >
              {pageSizes.map((size) => (
                <option key={size} value={size}>
                  {size}
                </option>
              ))}
            </select>
            <Icon
              name="expand_more"
              size={15}
              className="pointer-events-none absolute top-1/2 right-1.5 -translate-y-1/2 text-ink-faint"
            />
          </div>
        </div>
      ) : null}

      <div className="flex flex-none items-center gap-1">
        <PagerArrow
          icon="chevron_left"
          label="Page précédente"
          disabled={!hasPrev}
          onClick={() => onPageChange(page - 1)}
        />
        {numbers.map((number) => (
          <button
            key={number}
            type="button"
            aria-current={number === page ? "page" : undefined}
            onClick={() => onPageChange(number)}
            className={cn(
              "tabular h-pager min-w-pager rounded-control border px-[7px] text-note font-mid transition-colors duration-150",
              number === page
                ? "border-accent bg-accent text-on-accent"
                : "border-line bg-surface text-ink-muted hover:bg-neutral-tint",
            )}
          >
            {number}
          </button>
        ))}
        {page_count > 3 ? <span className="px-1 text-note text-ink-faint">…</span> : null}
        <PagerArrow
          icon="chevron_right"
          label="Page suivante"
          disabled={!hasNext}
          accent
          onClick={() => onPageChange(page + 1)}
        />
      </div>
    </nav>
  );
}

function PagerArrow({
  icon,
  label,
  disabled,
  accent = false,
  onClick,
}: {
  icon: IconName;
  label: string;
  disabled: boolean;
  /** La flèche « suivant » est en accent lorsqu'elle est active, comme dans les maquettes. */
  accent?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "flex size-pager items-center justify-center rounded-control border border-line transition-colors duration-150",
        disabled
          ? "cursor-not-allowed bg-neutral-tint text-ink-faint"
          : cn("bg-surface hover:bg-neutral-tint", accent ? "text-accent" : "text-ink-muted"),
      )}
    >
      <Icon name={icon} size={16} />
    </button>
  );
}

/**
 * Pagination compacte d'une colonne de Kanban : deux flèches de 24 px encadrant le compteur.
 */
export function ColumnPager({
  page,
  page_size,
  total,
  label,
  onPageChange,
}: {
  page: number;
  page_size: number;
  total: number;
  label: string;
  onPageChange: (page: number) => void;
}) {
  const { from, to, hasPrev, hasNext } = page_bounds(page, page_size, total);

  return (
    <div className="flex items-center justify-between gap-2 border-t border-line px-3 py-2">
      <ColumnArrow
        icon="chevron_left"
        label={`Page précédente de ${label}`}
        disabled={!hasPrev}
        onClick={() => onPageChange(page - 1)}
      />
      <span className="tabular text-meta text-ink-faint">
        {from}–{to} sur {total}
      </span>
      <ColumnArrow
        icon="chevron_right"
        label={`Page suivante de ${label}`}
        disabled={!hasNext}
        accent
        onClick={() => onPageChange(page + 1)}
      />
    </div>
  );
}

function ColumnArrow({
  icon,
  label,
  disabled,
  accent = false,
  onClick,
}: {
  icon: IconName;
  label: string;
  disabled: boolean;
  accent?: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "flex size-6 items-center justify-center rounded-pill border border-line transition-colors duration-150",
        disabled
          ? "cursor-not-allowed bg-neutral-tint text-ink-faint"
          : cn("bg-surface hover:bg-neutral-tint", accent ? "text-accent" : "text-ink-muted"),
      )}
    >
      <Icon name={icon} size={15} />
    </button>
  );
}
