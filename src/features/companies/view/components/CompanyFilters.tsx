import type { ReactNode } from "react";
import type { CompanyCriteria } from "../../viewmodel/useCompaniesViewModel";
import {
  CompanySizes,
  companySizeLabel,
  referenceLabel,
  useReferentials,
} from "@/features/referentials";
import {
  ActiveFilterChip,
  ClearFiltersButton,
  FilterBar,
  FilterGroup,
  FilterMenu,
  FilterOption,
  SearchInput,
} from "@/shared/ui";

/** Bascule un critère à choix unique : le resélectionner l'efface. */
function pick<T>(current: T | null, value: T): T | null {
  return current === value ? null : value;
}

/**
 * Barre d'outils du répertoire : recherche, filtres, chips, actions.
 *
 * Secteur, type et taille sont trois axes indépendants — une société peut être « ESN + PME »
 * comme « Association + TPE ».
 */
export function CompanyFilters({
  search,
  onSearch,
  criteria,
  count,
  total,
  onApply,
  onReset,
  actions,
}: {
  search: string;
  onSearch: (value: string) => void;
  criteria: CompanyCriteria;
  count: number;
  /** Total renvoyé par SQLite après application du filtre courant. */
  total: number | null;
  onApply: (values: CompanyCriteria) => void;
  onReset: () => void;
  actions?: ReactNode;
}) {
  const referentials = useReferentials();

  return (
    <FilterBar actions={actions}>
      <SearchInput
        variant="toolbar"
        value={search}
        onValueChange={onSearch}
        placeholder="Rechercher…"
      />
      <FilterMenu count={count}>
        <FilterGroup label="Secteur d'activité">
          {referentials.data.sectors.map((sector) => (
            <FilterOption
              key={sector.id}
              label={sector.name}
              selected={criteria.sector_id === sector.id}
              onSelect={() =>
                onApply({ ...criteria, sector_id: pick(criteria.sector_id, sector.id) })
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Type d'entreprise">
          {referentials.data.company_types.map((type) => (
            <FilterOption
              key={type.code}
              label={type.name}
              selected={criteria.company_type_id === type.code}
              onSelect={() =>
                onApply({
                  ...criteria,
                  company_type_id: pick(criteria.company_type_id, type.code),
                })
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Taille">
          {CompanySizes.map((size) => (
            <FilterOption
              key={size.value}
              label={size.label}
              selected={criteria.company_size === size.value}
              onSelect={() =>
                onApply({
                  ...criteria,
                  company_size: pick(criteria.company_size, size.value),
                })
              }
            />
          ))}
        </FilterGroup>
      </FilterMenu>

      {criteria.sector_id ? (
        <ActiveFilterChip
          field="Secteur"
          value={
            referentials.data.sectors.find((sector) => sector.id === criteria.sector_id)?.name ??
            criteria.sector_id
          }
          onRemove={() => onApply({ ...criteria, sector_id: null })}
        />
      ) : null}
      {criteria.company_type_id ? (
        <ActiveFilterChip
          field="Type"
          value={
            referenceLabel(referentials.data.company_types, criteria.company_type_id) ??
            criteria.company_type_id
          }
          onRemove={() => onApply({ ...criteria, company_type_id: null })}
        />
      ) : null}
      {criteria.company_size ? (
        <ActiveFilterChip
          field="Taille"
          value={companySizeLabel(criteria.company_size)}
          onRemove={() => onApply({ ...criteria, company_size: null })}
        />
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
