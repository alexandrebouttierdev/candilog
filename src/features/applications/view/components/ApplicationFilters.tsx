import { useEffect, useState, type ReactNode } from "react";
import { type ApplicationFilterValues } from "../../model/schemas/application-filter.schema";
import { Contracts, Statuses, contract_label } from "../../model/statuses";
import { FORMAT_DATE, versDateAffichee, versDateIso } from "@/shared/lib/dates";
import {
  ActiveFilterChip,
  ClearFiltersButton,
  FilterBar,
  FilterGroup,
  FilterMenu,
  FilterOption,
  SearchInput,
} from "@/shared/ui";
import { cn } from "@/shared/lib/cn";

function patch(
  filters: ApplicationFilterValues,
  update: Partial<ApplicationFilterValues>,
): ApplicationFilterValues {
  return { ...filters, ...update };
}

function periodValue(start: string | null, end: string | null): string | null {
  if (start && end) return `${versDateAffichee(start)} → ${versDateAffichee(end)}`;
  if (start) return `Depuis le ${versDateAffichee(start)}`;
  if (end) return `Jusqu'au ${versDateAffichee(end)}`;
  return null;
}

function CompactField({
  value,
  placeholder,
  onChange,
}: {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
}) {
  return (
    <input
      value={value}
      placeholder={placeholder}
      onChange={(event) => onChange(event.target.value)}
      className={cn(
        "h-[25px] min-h-[25px] w-full rounded-chip border border-control bg-fill px-2 text-label text-ink",
        "placeholder:text-ink-disabled focus:border-accent focus:outline-none",
      )}
    />
  );
}

/**
 * Toolbar du suivi : recherche 300 px, bouton Filtres, chips, actions.
 */
export function ApplicationFilters({
  search,
  onSearch,
  filters,
  count,
  onApply,
  onReset,
  actions,
}: {
  search: string;
  onSearch: (value: string) => void;
  filters: ApplicationFilterValues;
  count: number;
  onApply: (values: ApplicationFilterValues) => void;
  onReset: () => void;
  actions?: ReactNode;
}) {
  const [start, setStart] = useState("");
  const [end, setEnd] = useState("");

  useEffect(() => {
    setStart(filters.start_date ? versDateAffichee(filters.start_date) : "");
    setEnd(filters.end_date ? versDateAffichee(filters.end_date) : "");
  }, [filters.start_date, filters.end_date]);

  const commitDate = (raw: string, key: "start_date" | "end_date") => {
    const trimmed = raw.trim();
    if (trimmed === "") {
      onApply(patch(filters, { [key]: null }));
      return;
    }
    const iso = versDateIso(trimmed);
    if (iso) onApply(patch(filters, { [key]: iso }));
  };

  const period = periodValue(filters.start_date, filters.end_date);

  return (
    <FilterBar actions={actions}>
      <SearchInput
        variant="toolbar"
        value={search}
        onValueChange={onSearch}
        placeholder="Rechercher…"
      />
      <FilterMenu count={count}>
        <FilterGroup label="Statut">
          {Statuses.map((status) => (
            <FilterOption
              key={status.value}
              label={status.label}
              selected={filters.status === status.value}
              onSelect={() =>
                onApply(
                  patch(filters, {
                    status: filters.status === status.value ? null : status.value,
                  }),
                )
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Contrat">
          {Contracts.map((contract) => (
            <FilterOption
              key={contract}
              label={contract_label(contract)}
              selected={filters.contract === contract}
              onSelect={() =>
                onApply(
                  patch(filters, {
                    contract: filters.contract === contract ? null : contract,
                  }),
                )
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Poste">
          <CompactField
            value={filters.job_title}
            placeholder="Développeur…"
            onChange={(job_title) => onApply(patch(filters, { job_title }))}
          />
        </FilterGroup>
        <FilterGroup label="Ville">
          <CompactField
            value={filters.city}
            placeholder="Rennes…"
            onChange={(city) => onApply(patch(filters, { city }))}
          />
        </FilterGroup>
        <FilterGroup label="Période">
          <CompactField
            value={start}
            placeholder={`${FORMAT_DATE} — début`}
            onChange={(value) => {
              setStart(value);
              commitDate(value, "start_date");
            }}
          />
          <CompactField
            value={end}
            placeholder={`${FORMAT_DATE} — fin`}
            onChange={(value) => {
              setEnd(value);
              commitDate(value, "end_date");
            }}
          />
        </FilterGroup>
      </FilterMenu>

      {filters.status ? (
        <ActiveFilterChip
          field="Statut"
          value={Statuses.find((status) => status.value === filters.status)?.label ?? filters.status}
          onRemove={() => onApply(patch(filters, { status: null }))}
        />
      ) : null}
      {filters.contract ? (
        <ActiveFilterChip
          field="Contrat"
          value={contract_label(filters.contract)}
          onRemove={() => onApply(patch(filters, { contract: null }))}
        />
      ) : null}
      {filters.job_title ? (
        <ActiveFilterChip
          field="Poste"
          value={filters.job_title}
          onRemove={() => onApply(patch(filters, { job_title: "" }))}
        />
      ) : null}
      {filters.city ? (
        <ActiveFilterChip
          field="Ville"
          value={filters.city}
          onRemove={() => onApply(patch(filters, { city: "" }))}
        />
      ) : null}
      {period ? (
        <ActiveFilterChip
          field="Période"
          value={period}
          onRemove={() => onApply(patch(filters, { start_date: null, end_date: null }))}
        />
      ) : null}
      {count > 0 ? <ClearFiltersButton onClick={onReset} /> : null}
    </FilterBar>
  );
}
