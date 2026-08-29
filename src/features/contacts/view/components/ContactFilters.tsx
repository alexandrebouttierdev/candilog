import type { ReactNode } from "react";
import { Roles } from "../../model/roles";
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
 * Toolbar du réseau : recherche 300 px, bouton Filtres, chips, actions.
 */
export function ContactFilters({
  search,
  onSearch,
  tracking_role,
  count,
  total,
  onSelectRole,
  onReset,
  actions,
}: {
  search: string;
  onSearch: (value: string) => void;
  tracking_role: string | null;
  count: number;
  /** Total renvoyé par SQLite après application du filtre courant. */
  total: number | null;
  onSelectRole: (value: string | null) => void;
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
        <FilterGroup label="Rôle">
          {Roles.map((role) => (
            <FilterOption
              key={role}
              label={role}
              selected={tracking_role === role}
              onSelect={() => onSelectRole(tracking_role === role ? null : role)}
            />
          ))}
        </FilterGroup>
      </FilterMenu>

      {tracking_role ? (
        <ActiveFilterChip
          field="Rôle"
          value={tracking_role}
          onRemove={() => onSelectRole(null)}
        />
      ) : null}
      {count > 0 ? <ClearFiltersButton onClick={onReset} /> : null}
      {total !== null ? (
        <p className="tabular text-note font-semibold text-ink" aria-live="polite">
          {total} contact{total === 1 ? "" : "s"}
        </p>
      ) : null}
    </FilterBar>
  );
}
