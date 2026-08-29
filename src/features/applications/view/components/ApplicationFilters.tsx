import { useState, type ReactNode } from "react";
import { type ApplicationFilterValues } from "../../model/schemas/application-filter.schema";
import { Contracts, Statuses, contract_label } from "../../model/statuses";
import { FORMAT_DATE, versDateAffichee, versDateIso } from "@/shared/lib/dates";
import {
  ActiveFilterChip,
  ClearFiltersButton,
  DateInput,
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

function toggle<T>(values: readonly T[], value: T): T[] {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
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

const ERREUR_DATE = `Date invalide — format attendu ${FORMAT_DATE}.`;

function messageDate(raw: string, auBlur: boolean): string | undefined {
  const trimmed = raw.trim();
  if (trimmed === "" || versDateIso(trimmed) !== null) return undefined;
  if (auBlur || /^\d{2}-\d{2}-\d{4}$/.test(trimmed)) return ERREUR_DATE;
  return undefined;
}

function affichee(iso: string | null): string {
  return iso ? versDateAffichee(iso) : "";
}

/**
 * Toolbar du suivi : recherche 300 px, bouton Filtres, chips, actions.
 */
export function ApplicationFilters({
  search,
  onSearch,
  filters,
  count,
  total,
  onApply,
  onReset,
  actions,
}: {
  search: string;
  onSearch: (value: string) => void;
  filters: ApplicationFilterValues;
  count: number;
  /** Total renvoyé par SQLite après application du filtre courant. */
  total: number | null;
  onApply: (values: ApplicationFilterValues) => void;
  onReset: () => void;
  actions?: ReactNode;
}) {
  const [start, setStart] = useState(() => affichee(filters.start_date));
  const [end, setEnd] = useState(() => affichee(filters.end_date));
  const [startError, setStartError] = useState<string | undefined>();
  const [endError, setEndError] = useState<string | undefined>();
  const [bornes, setBornes] = useState({
    start: filters.start_date,
    end: filters.end_date,
  });

  if (filters.start_date !== bornes.start || filters.end_date !== bornes.end) {
    setBornes({ start: filters.start_date, end: filters.end_date });
    setStart(affichee(filters.start_date));
    setEnd(affichee(filters.end_date));
    setStartError(undefined);
    setEndError(undefined);
  }

  const commitDate = (
    raw: string,
    key: "start_date" | "end_date",
    setError: (message: string | undefined) => void,
    auBlur: boolean,
  ) => {
    const trimmed = raw.trim();
    if (trimmed === "") {
      setError(undefined);
      onApply(patch(filters, { [key]: null }));
      return;
    }
    const iso = versDateIso(trimmed);
    if (iso) {
      setError(undefined);
      onApply(patch(filters, { [key]: iso }));
      return;
    }
    setError(messageDate(raw, auBlur));
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
              selected={filters.status.includes(status.value)}
              onSelect={() =>
                onApply(patch(filters, { status: toggle(filters.status, status.value) }))
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Contrat">
          {Contracts.map((contract) => (
            <FilterOption
              key={contract}
              label={contract_label(contract)}
              selected={filters.contract.includes(contract)}
              onSelect={() =>
                onApply(patch(filters, { contract: toggle(filters.contract, contract) }))
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
          <div className="flex w-full flex-col gap-1.5">
            <DateInput
              dense
              className="w-full"
              aria-label="Début de période"
              placeholder={`${FORMAT_DATE} — début`}
              value={start}
              invalid={Boolean(startError)}
              onChange={(event) => {
                setStart(event.target.value);
                commitDate(event.target.value, "start_date", setStartError, false);
              }}
              onBlur={(event) =>
                commitDate(event.target.value, "start_date", setStartError, true)
              }
            />
            <DateInput
              dense
              className="w-full"
              aria-label="Fin de période"
              placeholder={`${FORMAT_DATE} — fin`}
              value={end}
              invalid={Boolean(endError)}
              onChange={(event) => {
                setEnd(event.target.value);
                commitDate(event.target.value, "end_date", setEndError, false);
              }}
              onBlur={(event) =>
                commitDate(event.target.value, "end_date", setEndError, true)
              }
            />
            {startError || endError ? (
              <p className="text-meta leading-[1.45] text-danger">{startError ?? endError}</p>
            ) : null}
          </div>
        </FilterGroup>
      </FilterMenu>

      {filters.status.map((value) => (
        <ActiveFilterChip
          key={`status-${value}`}
          field="Statut"
          value={Statuses.find((status) => status.value === value)?.label ?? value}
          onRemove={() =>
            onApply(patch(filters, { status: filters.status.filter((item) => item !== value) }))
          }
        />
      ))}
      {filters.contract.map((value) => (
        <ActiveFilterChip
          key={`contract-${value}`}
          field="Contrat"
          value={contract_label(value)}
          onRemove={() =>
            onApply(
              patch(filters, { contract: filters.contract.filter((item) => item !== value) }),
            )
          }
        />
      ))}
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
      {total !== null ? (
        <p className="tabular text-note font-semibold text-ink" aria-live="polite">
          {total} candidature{total === 1 ? "" : "s"}
        </p>
      ) : null}
    </FilterBar>
  );
}
