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
  criteres,
  count,
  total,
  onApply,
  onReset,
  actions,
}: {
  search: string;
  onSearch: (value: string) => void;
  criteres: CompanyCriteria;
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
              selected={criteres.sector_id === sector.id}
              onSelect={() =>
                onApply({ ...criteres, sector_id: pick(criteres.sector_id, sector.id) })
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Type d'entreprise">
          {referentials.data.company_types.map((type) => (
            <FilterOption
              key={type.code}
              label={type.name}
              selected={criteres.company_type_id === type.code}
              onSelect={() =>
                onApply({
                  ...criteres,
                  company_type_id: pick(criteres.company_type_id, type.code),
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
              selected={criteres.company_size === size.value}
              onSelect={() =>
                onApply({
                  ...criteres,
                  company_size: pick(criteres.company_size, size.value),
                })
              }
            />
          ))}
        </FilterGroup>
      </FilterMenu>

      {criteres.sector_id ? (
        <ActiveFilterChip
          field="Secteur"
          value={
            referentials.data.sectors.find((sector) => sector.id === criteres.sector_id)?.name ??
            criteres.sector_id
          }
          onRemove={() => onApply({ ...criteres, sector_id: null })}
        />
      ) : null}
      {criteres.company_type_id ? (
        <ActiveFilterChip
          field="Type"
          value={
            referenceLabel(referentials.data.company_types, criteres.company_type_id) ??
            criteres.company_type_id
          }
          onRemove={() => onApply({ ...criteres, company_type_id: null })}
        />
      ) : null}
      {criteres.company_size ? (
        <ActiveFilterChip
          field="Taille"
          value={companySizeLabel(criteres.company_size)}
          onRemove={() => onApply({ ...criteres, company_size: null })}
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
