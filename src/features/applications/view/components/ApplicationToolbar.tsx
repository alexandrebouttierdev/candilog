import { useCallback, useRef, useState } from "react";
import type { ReactNode } from "react";
import { cn } from "@/shared/lib/cn";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { Kbd } from "@/shared/ui";
import type { MenuAnchor } from "@/shared/ui";
import type { ApplicationFilterValues } from "../../model/schemas/application-filter.schema";
import { chipsOf, removeField, toggleExcluded } from "../../model/filterFields";
import { useFilterOptions } from "../../viewmodel/useFilterOptions";
import { ApplicationFilterMenu } from "./ApplicationFilterMenu";

/**
 * Barre d'outils de Candidatures (`screens/02-applications-list.png`), 38 px : puces des
 * critères actifs, « + Filtre » (`F`), « Tout effacer », puis à droite le décompte filtré,
 * la recherche (`/`) et les actions de l'écran.
 *
 * Un clic sur une puce inverse sa condition (« est » ↔ « n'est pas ») ; sa croix la retire.
 */
export function ApplicationToolbar({
  filters,
  onApply,
  onReset,
  search,
  onSearch,
  count,
  actions,
}: {
  filters: ApplicationFilterValues;
  onApply: (filters: ApplicationFilterValues) => void;
  onReset: () => void;
  search: string;
  onSearch: (value: string) => void;
  /** « 3 / 24 » quand un critère réduit la liste ; `null` sinon. */
  count: { filtered: number; total: number } | null;
  actions: ReactNode;
}) {
  const [menuAnchor, setMenuAnchor] = useState<MenuAnchor | null>(null);
  const addButton = useRef<HTMLButtonElement>(null);
  const searchInput = useRef<HTMLInputElement>(null);
  const { label } = useFilterOptions(filters.company_id);
  const chips = chipsOf(filters, label);

  // Stable : le menu reprend le focus quand `onClose` change, ce qui le volerait au champ
  // de saisie de son second niveau à chaque frappe.
  const closeMenu = useCallback(() => setMenuAnchor(null), []);
  const openMenu = () => {
    const rect = addButton.current?.getBoundingClientRect();
    if (rect) setMenuAnchor(rect);
  };
  useShortcut("f", openMenu);
  useShortcut("/", () => searchInput.current?.focus());

  return (
    <div className="flex h-toolbar flex-none items-center gap-[7px] border-b border-bd-soft px-3.5">
      <div className="flex min-w-0 flex-1 items-center gap-[7px] overflow-x-auto">
        {chips.map((chip) => (
          <span
            key={chip.key}
            className="inline-flex h-[23px] max-w-[320px] flex-none items-center gap-1.5 rounded-r6 bg-elev pr-1 pl-2 text-small text-tx-2"
          >
            <button
              type="button"
              disabled={!chip.negatable}
              title={chip.negatable ? "Inverser la condition" : undefined}
              aria-label={`${chip.field} ${chip.op} ${chip.value}${chip.negatable ? ", inverser la condition" : ""}`}
              onClick={() => onApply(toggleExcluded(filters, chip.key))}
              className="flex min-w-0 items-center gap-1.5 hover:text-tx disabled:cursor-default"
            >
              <span className="flex-none">{chip.field}</span>
              <span className="flex-none text-tx-4">{chip.op}</span>
              <span className="truncate font-medium text-tx">{chip.value}</span>
            </button>
            <button
              type="button"
              aria-label={`Retirer le filtre ${chip.field}`}
              onClick={() => onApply(removeField(filters, chip.key))}
              className="flex size-4 flex-none items-center justify-center rounded-r4 text-[10px] text-tx-5 hover:bg-hover hover:text-tx-2"
            >
              ✕
            </button>
          </span>
        ))}
        <button
          ref={addButton}
          type="button"
          aria-haspopup="menu"
          aria-expanded={menuAnchor !== null}
          aria-keyshortcuts="F"
          onClick={openMenu}
          className={cn(
            "inline-flex h-[23px] flex-none items-center gap-[5px] rounded-r6 px-2 text-small text-tx-5 hover:text-tx-3",
            menuAnchor && "bg-elev",
          )}
        >
          <span aria-hidden className="text-tiny">
            +
          </span>
          Filtre
          <Kbd shortcut="f" tone="ghost" decorative />
        </button>
        {chips.length > 0 ? (
          <button
            type="button"
            onClick={onReset}
            className="inline-flex h-[23px] flex-none items-center rounded-r6 px-2 text-small text-tx-5 hover:text-tx-3"
          >
            Tout effacer
          </button>
        ) : null}
      </div>

      <div className="flex flex-none items-center gap-[7px]">
        {count ? (
          <span className="font-mono text-caps whitespace-nowrap text-tx-5" aria-live="polite">
            {count.filtered} / {count.total}
          </span>
        ) : null}
        <label
          className={cn(
            "flex h-[23px] w-[150px] items-center gap-[7px] rounded-r6 border px-[9px] text-small transition-[width] duration-100 focus-within:w-[200px]",
            "border-transparent bg-elev focus-within:border-ac focus-within:bg-panel",
          )}
        >
          <span aria-hidden className="flex-none text-tiny text-tx-5">
            ⌕
          </span>
          <input
            ref={searchInput}
            type="search"
            aria-label="Rechercher une candidature"
            aria-keyshortcuts="/"
            placeholder="Rechercher"
            value={search}
            onChange={(event) => onSearch(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.stopPropagation();
                if (search) onSearch("");
                else event.currentTarget.blur();
              }
            }}
            className="peer min-w-0 flex-1 bg-transparent text-small text-tx outline-none placeholder:text-tx-5 [&::-webkit-search-cancel-button]:hidden"
          />
          {search ? null : <Kbd shortcut="/" tone="ghost" decorative className="peer-focus:hidden" />}
        </label>
        {actions}
      </div>

      <ApplicationFilterMenu
        open={menuAnchor !== null}
        anchor={menuAnchor}
        filters={filters}
        onApply={onApply}
        onClose={closeMenu}
      />
    </div>
  );
}
