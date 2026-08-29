import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/** Description d'une colonne : en-tête, rendu de cellule, tri éventuel. */
export interface Column<TRow, TSortKey extends string = string> {
  /** Id stable de la colonne, utilisé comme clé React. */
  readonly key: string;
  readonly header: string;
  /** Clé de tri envoyée au backend ; absente si la colonne n'est pas triable. */
  readonly sort_key?: TSortKey;
  /**
   * Part de la largeur disponible, comme les `fr` des grilles des maquettes
   * (« 2.2fr 1.3fr 0.9fr … »). Vaut 1 par défaut.
   */
  readonly grow?: number;
  /** Aligner à droite : réservé aux nombres et aux dates. */
  readonly numeric?: boolean;
  readonly render: (row: TRow) => ReactNode;
}

export interface SortState<TSortKey extends string> {
  readonly key: TSortKey;
  readonly direction: "asc" | "desc";
}

/**
 * Table dense du guide SPECDESIGN.
 *
 * Grid CSS plutôt que `<table>` : les maquettes répartissent les colonnes en fractions
 * (`2.2fr 1.3fr 0.9fr 1.1fr 1fr 0.7fr`) et alignent verticalement des cellules à deux
 * lignes ; une table HTML rendrait les mêmes proportions dépendantes du contenu. Les rôles
 * ARIA de tableau sont posés à la main pour ne rien perdre en restitution.
 *
 * Le tri est **délégué** : le composant émet la colonne demandée et affiche la direction
 * courante, mais ne trie rien lui-même. Trier ici ne trierait que la page affichée, ce qui
 * donnerait un ordre faux dès que les données dépassent une page — or la pagination est la
 * règle au-delà de 50 éléments.
 */
export function DataTable<TRow, TSortKey extends string = string>({
  columns,
  rows,
  row_key,
  sort,
  onSortChange,
  onRowClick,
  isSelected,
  header,
  empty_state,
  footer,
}: {
  columns: readonly Column<TRow, TSortKey>[];
  rows: readonly TRow[];
  row_key: (row: TRow) => string;
  sort?: SortState<TSortKey>;
  onSortChange?: (key: TSortKey) => void;
  onRowClick?: (row: TRow) => void;
  isSelected?: (row: TRow) => boolean;
  /** Bandeau titré au-dessus des en-têtes de colonnes — un `CardHeader`. */
  header?: ReactNode;
  /** Affiché à la place du corps lorsque `rows` est vide. */
  empty_state?: ReactNode;
  /** Pied du tableau, à l'intérieur de la carte — typiquement un `Pager`. */
  footer?: ReactNode;
}) {
  const template = columns.map((column) => `${column.grow ?? 1}fr`).join(" ");

  return (
    <div
      role="table"
      className="min-w-0 overflow-hidden rounded-card border border-line bg-surface"
    >
      {header}

      <div role="rowgroup">
        <div
          role="row"
          style={{ gridTemplateColumns: template }}
          className="grid h-[34px] items-center bg-surface-elevated px-3.5"
        >
          {columns.map((column) => {
            const sortable = column.sort_key !== undefined && onSortChange !== undefined;
            const activeSort = sort && sort.key === column.sort_key ? sort : undefined;
            const content = (
              <>
                {column.header}
                {sortable ? (
                  <Icon
                    name={
                      activeSort
                        ? activeSort.direction === "asc"
                          ? "arrow_upward"
                          : "arrow_downward"
                        : "unfold_more"
                    }
                    size={13}
                    className={activeSort ? "text-accent" : "text-ink-faint"}
                  />
                ) : null}
              </>
            );

            return (
              <div
                key={column.key}
                role="columnheader"
                aria-sort={
                  activeSort
                    ? activeSort.direction === "asc"
                      ? "ascending"
                      : "descending"
                    : undefined
                }
                className={cn(
                  "min-w-0 text-eyebrow uppercase tracking-[0.05em] text-ink-disabled",
                  column.numeric && "text-right",
                )}
              >
                {sortable ? (
                  <button
                    type="button"
                    onClick={() => onSortChange(column.sort_key as TSortKey)}
                    className={cn(
                      "inline-flex items-center gap-1 transition-colors duration-150 hover:text-ink",
                      column.numeric && "flex-row-reverse",
                    )}
                  >
                    {content}
                  </button>
                ) : (
                  content
                )}
              </div>
            );
          })}
        </div>
      </div>

      {rows.length === 0 && empty_state ? (
        empty_state
      ) : (
        <div role="rowgroup">
          {rows.map((row) => (
            <div
              key={row_key(row)}
              role="row"
              onClick={onRowClick ? () => onRowClick(row) : undefined}
              tabIndex={onRowClick ? 0 : undefined}
              onKeyDown={
                onRowClick
                  ? (event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        onRowClick(row);
                      }
                    }
                  : undefined
              }
              style={{ gridTemplateColumns: template }}
              className={cn(
                "grid h-row items-center border-t border-field px-3.5",
                onRowClick && "cursor-pointer transition-colors duration-hover",
                isSelected?.(row)
                  ? "row-selected"
                  : onRowClick && "hover:bg-surface-hover",
              )}
            >
              {columns.map((column) => (
                <div
                  key={column.key}
                  role="cell"
                  className={cn("min-w-0", column.numeric && "tabular text-right")}
                >
                  {column.render(row)}
                </div>
              ))}
            </div>
          ))}
        </div>
      )}

      {footer}
    </div>
  );
}

/**
 * Cellule « intitulé » des tableaux : pastille d'initiales, titre 13 px et sous-titre.
 *
 * Présente à l'identique dans le tableau du Tracking, celui du Table de bord et la liste
 * des candidatures à relancer — d'où sa place ici plutôt que recopiée trois fois.
 */
export function CellIdentity({
  initials,
  title,
  subtitle,
}: {
  initials: string;
  title: string;
  subtitle?: ReactNode;
}) {
  return (
    <div className="flex min-w-0 items-center gap-[11px]">
      <span
        aria-hidden="true"
        className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-meta font-strong text-ink-muted"
      >
        {initials}
      </span>
      <div className="min-w-0">
        <div className="truncate text-item font-mid text-ink">{title}</div>
        {subtitle ? (
          <div className="mt-px truncate text-label text-ink-faint">{subtitle}</div>
        ) : null}
      </div>
    </div>
  );
}
