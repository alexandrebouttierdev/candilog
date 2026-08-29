import type { ReactNode } from "react";
import {
  ActiveFilterChip,
  ClearFiltersButton,
  FilterBar,
  FilterGroup,
  FilterMenu,
  FilterOption,
  SearchInput,
} from "@/shared/ui";

/**
 * Toolbar du répertoire : recherche 300 px, bouton Filtres, chips, actions.
 */
export function CompanyFilters({
  search,
  onSearch,
  company_type,
  types,
  count,
  total,
  onSelectType,
  onReset,
  actions,
}: {
  search: string;
  onSearch: (value: string) => void;
  company_type: string | null;
  types: readonly string[];
  count: number;
  /** Total renvoyé par SQLite après application du filtre courant. */
  total: number | null;
  onSelectType: (value: string | null) => void;
  onReset: () => void;
  actions?: ReactNode;
}) {
  return (
    <FilterBar actions={actions}>
      <SearchInput
        variant="toolbar"
        value={search}
        onValueChange={onSearch}
        placeholder="Rechercher…"
      />
      <FilterMenu count={count}>
        <FilterGroup label="Type">
          {types.map((type) => (
            <FilterOption
              key={type}
              label={type}
              selected={company_type === type}
              onSelect={() => onSelectType(company_type === type ? null : type)}
            />
          ))}
        </FilterGroup>
      </FilterMenu>

      {company_type ? (
        <ActiveFilterChip field="Type" value={company_type} onRemove={() => onSelectType(null)} />
      ) : null}
      {count > 0 ? <ClearFiltersButton onClick={onReset} /> : null}
      {total !== null ? (
        <p className="tabular text-note font-semibold text-ink" aria-live="polite">
          {total} entreprise{total === 1 ? "" : "s"}
        </p>
      ) : null}
    </FilterBar>
  );
}
