import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/** Bornes d'une page, calculées une fois pour l'affichage et pour les tests. */
export function pageBounds(page: number, pageSize: number, total: number) {
  const pageCount = Math.max(1, Math.ceil(total / pageSize));
  const from = total === 0 ? 0 : (page - 1) * pageSize + 1;
  const to = Math.min(page * pageSize, total);
  return { pageCount, from, to, hasPrev: page > 1, hasNext: page < pageCount };
}

/**
 * Pagination réutilisable.
 *
 * La pagination est **côté données** : le composant ne reçoit jamais la collection
 * complète, seulement la page courante et le total, et le ViewModel ne demande au backend
 * que la page affichée. Le guide exige de ne matérialiser qu'une page au-delà de 50
 * éléments ; passer la liste entière ici l'aurait rendu impossible par construction.
 */
export function Pager({
  page,
  pageSize,
  total,
  label,
  onPageChange,
}: {
  page: number;
  pageSize: number;
  total: number;
  /** Nom de ce qui est compté, au pluriel : « candidatures », « entreprises ». */
  label: string;
  onPageChange: (page: number) => void;
}) {
  const { pageCount, from, to, hasPrev, hasNext } = pageBounds(page, pageSize, total);

  // Les trois premiers numéros puis une ellipse : le guide affiche un repère de position,
  // pas un index complet — au-delà, le compteur textuel est plus lisible qu'une rangée
  // de numéros.
  const numbers = Array.from({ length: Math.min(pageCount, 3) }, (_, index) => index + 1);

  return (
    <nav
      aria-label={`Pagination des ${label}`}
      className="flex flex-none items-center justify-between gap-3 border-t border-line bg-surface-alt px-4 py-2.5"
    >
      <p className="tabular text-meta text-ink-muted">
        {from}–{to} sur {total} {label}
      </p>
      <div className="flex items-center gap-1">
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
              "tabular size-7 rounded-button border text-meta transition-colors duration-150",
              number === page
                ? "border-accent bg-accent text-white"
                : "border-line bg-surface text-ink-muted hover:bg-neutral-tint",
            )}
          >
            {number}
          </button>
        ))}
        {pageCount > 3 ? <span className="px-1 text-meta text-ink-faint">…</span> : null}
        <PagerArrow
          icon="chevron_right"
          label="Page suivante"
          disabled={!hasNext}
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
  onClick,
}: {
  icon: string;
  label: string;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "flex size-7 items-center justify-center rounded-button border transition-colors duration-150",
        disabled
          ? "border-line bg-neutral-tint text-ink-faint"
          : "border-line bg-surface text-ink-muted hover:bg-neutral-tint",
      )}
    >
      <Icon name={icon} size={16} />
    </button>
  );
}
