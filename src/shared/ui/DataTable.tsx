import type { ReactNode } from "react";
import { Icon } from "./Icon";
import { cn } from "@/shared/lib/cn";

/** Description d'une colonne : en-tête, rendu de cellule, tri éventuel. */
export interface Column<TRow, TSortKey extends string = string> {
  /** Identifiant stable de la colonne, utilisé comme clé React. */
  readonly key: string;
  readonly header: string;
  /** Clé de tri envoyée au backend ; absente si la colonne n'est pas triable. */
  readonly sortKey?: TSortKey;
  /** Largeur CSS de la colonne ; `undefined` laisse la colonne s'étirer. */
  readonly width?: string;
  /** Aligner à droite : réservé aux nombres et aux dates. */
  readonly numeric?: boolean;
  readonly render: (row: TRow) => ReactNode;
}

export interface SortState<TSortKey extends string> {
  readonly key: TSortKey;
  readonly direction: "asc" | "desc";
}

/**
 * Tableau dense du guide SPECDESIGN : lignes de 44 px, en-tête triable, ligne cliquable.
 *
 * Le tri est **délégué** : le composant émet la colonne demandée et affiche la direction
 * courante, mais ne trie rien lui-même. Trier ici ne trierait que la page affichée, ce qui
 * donnerait un ordre faux dès que les données dépassent une page — or la pagination est la
 * règle au-delà de 50 éléments.
 */
export function DataTable<TRow, TSortKey extends string = string>({
  columns,
  rows,
  rowKey,
  sort,
  onSortChange,
  onRowClick,
  emptyState,
}: {
  columns: readonly Column<TRow, TSortKey>[];
  rows: readonly TRow[];
  rowKey: (row: TRow) => string;
  sort?: SortState<TSortKey>;
  onSortChange?: (key: TSortKey) => void;
  onRowClick?: (row: TRow) => void;
  /** Affiché à la place du corps lorsque `rows` est vide. */
  emptyState?: ReactNode;
}) {
  if (rows.length === 0 && emptyState) {
    return (
      <div className="overflow-hidden rounded-card border border-line bg-surface">
        <TableHead columns={columns} sort={sort} onSortChange={onSortChange} />
        {emptyState}
      </div>
    );
  }

  return (
    <div className="overflow-x-auto rounded-card border border-line bg-surface">
      <table className="w-full min-w-[720px] border-collapse text-body">
        <TableHead columns={columns} sort={sort} onSortChange={onSortChange} asTable />
        <tbody>
          {rows.map((row) => (
            <tr
              key={rowKey(row)}
              onClick={onRowClick ? () => onRowClick(row) : undefined}
              tabIndex={onRowClick ? 0 : undefined}
              onKeyDown={
                onRowClick
                  ? (event) => {
                      if (event.key === "Enter") onRowClick(row);
                    }
                  : undefined
              }
              className={cn(
                "h-row border-b border-line last:border-b-0",
                onRowClick && "cursor-pointer transition-colors duration-150 hover:bg-neutral-tint",
              )}
            >
              {columns.map((column) => (
                <td
                  key={column.key}
                  className={cn(
                    "px-4 align-middle",
                    column.numeric && "tabular text-right",
                  )}
                  style={column.width ? { width: column.width } : undefined}
                >
                  {column.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function TableHead<TRow, TSortKey extends string>({
  columns,
  sort,
  onSortChange,
  asTable = false,
}: {
  columns: readonly Column<TRow, TSortKey>[];
  // `| undefined` explicite : `exactOptionalPropertyTypes` distingue « propriété absente »
  // de « propriété valant undefined », et le parent transmet ici la seconde forme.
  sort?: SortState<TSortKey> | undefined;
  onSortChange?: ((key: TSortKey) => void) | undefined;
  asTable?: boolean;
}) {
  const cells = columns.map((column) => {
    const sortable = column.sortKey !== undefined && onSortChange !== undefined;
    // Lié à une constante locale : le rétrécissement de type sur `sort?.key` ne se propage
    // pas jusqu'aux lectures de `sort.direction` plus bas.
    const activeSort = sort && sort.key === column.sortKey ? sort : undefined;
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

    const className = cn(
      "px-4 py-2.5 text-left text-eyebrow uppercase text-ink-faint",
      column.numeric && "text-right",
    );

    if (!asTable) {
      return (
        <div key={column.key} className={className}>
          {content}
        </div>
      );
    }

    return (
      <th
        key={column.key}
        scope="col"
        aria-sort={
          activeSort
            ? activeSort.direction === "asc"
              ? "ascending"
              : "descending"
            : undefined
        }
        className={className}
        style={column.width ? { width: column.width } : undefined}
      >
        {sortable ? (
          <button
            type="button"
            onClick={() => onSortChange(column.sortKey as TSortKey)}
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
      </th>
    );
  });

  if (!asTable) {
    return <div className="flex border-b border-line bg-surface-alt">{cells}</div>;
  }

  return (
    <thead className="border-b border-line bg-surface-alt">
      <tr>{cells}</tr>
    </thead>
  );
}
