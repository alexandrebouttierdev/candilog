import { useState, type ReactNode } from "react";
import { type ApplicationFilterValues } from "../../model/schemas/application-filter.schema";
import { Statuses } from "../../model/statuses";
import {
  ApplicationTypes,
  CompanySizes,
  WeeklyWorkSchedules,
  formatHours,
  referenceLabel,
  useReferentials,
} from "@/features/referentials";
import type { ReferenceItem } from "@/features/referentials";
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

/** Libellé de l'amplitude horaire retenue, ou `null` si aucune borne n'est posée. */
function hoursValue(min: number | null, max: number | null): string | null {
  if (min !== null && max !== null) return `${formatHours(min)} → ${formatHours(max)} h`;
  if (min !== null) return `À partir de ${formatHours(min)} h`;
  if (max !== null) return `Jusqu'à ${formatHours(max)} h`;
  return null;
}

function CompactField({
  value,
  placeholder,
  inputMode,
  onChange,
}: {
  value: string;
  placeholder: string;
  inputMode?: "decimal";
  onChange: (value: string) => void;
}) {
  return (
    <input
      value={value}
      placeholder={placeholder}
      inputMode={inputMode}
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

/** Nombre saisi au clavier, virgule décimale acceptée ; `null` si vide ou invalide. */
function versHeures(raw: string): number | null {
  const trimmed = raw.trim();
  if (trimmed === "") return null;
  const hours = Number(trimmed.replace(",", "."));
  return Number.isFinite(hours) && hours > 0 ? hours : null;
}

/**
 * Barre d'outils du suivi : recherche, menu de filtres, chips, actions.
 *
 * Les options des quatre référentiels viennent de la base, jamais d'une liste écrite ici :
 * une seconde copie divergerait au premier ajout.
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
  const referentials = useReferentials();
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

  /** Groupe de cases à cocher alimenté par un référentiel de la base. */
  const groupeReferentiel = (
    label: string,
    items: readonly ReferenceItem[],
    selection: readonly string[],
    key: "contract_type_code" | "professional_domain_id" | "company_type_id",
  ) => (
    <FilterGroup label={label}>
      {items.map((item) => (
        <FilterOption
          key={item.code}
          label={item.name}
          selected={selection.includes(item.code)}
          onSelect={() => onApply(patch(filters, { [key]: toggle(selection, item.code) }))}
        />
      ))}
    </FilterGroup>
  );

  const period = periodValue(filters.start_date, filters.end_date);
  const hours = hoursValue(filters.min_weekly_hours, filters.max_weekly_hours);

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
        <FilterGroup label="Type de candidature">
          {ApplicationTypes.map((type) => (
            <FilterOption
              key={type.value}
              label={type.label}
              selected={filters.application_type.includes(type.value)}
              onSelect={() =>
                onApply(
                  patch(filters, {
                    application_type: toggle(filters.application_type, type.value),
                  }),
                )
              }
            />
          ))}
        </FilterGroup>
        {groupeReferentiel(
          "Contrat",
          referentials.data.contract_types,
          filters.contract_type_code,
          "contract_type_code",
        )}
        {groupeReferentiel(
          "Domaine professionnel",
          referentials.data.professional_domains,
          filters.professional_domain_id,
          "professional_domain_id",
        )}
        {groupeReferentiel(
          "Type d'entreprise",
          referentials.data.company_types,
          filters.company_type_id,
          "company_type_id",
        )}
        <FilterGroup label="Secteur d'activité">
          {referentials.data.sectors.map((sector) => (
            <FilterOption
              key={sector.id}
              label={sector.name}
              selected={filters.sector_id.includes(sector.id)}
              onSelect={() =>
                onApply(patch(filters, { sector_id: toggle(filters.sector_id, sector.id) }))
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Taille d'entreprise">
          {CompanySizes.map((size) => (
            <FilterOption
              key={size.value}
              label={size.label}
              selected={filters.company_size.includes(size.value)}
              onSelect={() =>
                onApply(
                  patch(filters, { company_size: toggle(filters.company_size, size.value) }),
                )
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Durée hebdomadaire">
          {WeeklyWorkSchedules.map((schedule) => (
            <FilterOption
              key={schedule.value}
              label={schedule.label}
              selected={filters.weekly_work_schedule.includes(schedule.value)}
              onSelect={() =>
                onApply(
                  patch(filters, {
                    weekly_work_schedule: toggle(
                      filters.weekly_work_schedule,
                      schedule.value,
                    ),
                  }),
                )
              }
            />
          ))}
        </FilterGroup>
        <FilterGroup label="Heures par semaine">
          <div className="flex w-full items-center gap-1.5">
            <CompactField
              value={filters.min_weekly_hours === null ? "" : String(filters.min_weekly_hours)}
              placeholder="min"
              inputMode="decimal"
              onChange={(value) =>
                onApply(patch(filters, { min_weekly_hours: versHeures(value) }))
              }
            />
            <CompactField
              value={filters.max_weekly_hours === null ? "" : String(filters.max_weekly_hours)}
              placeholder="max"
              inputMode="decimal"
              onChange={(value) =>
                onApply(patch(filters, { max_weekly_hours: versHeures(value) }))
              }
            />
          </div>
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
              onBlur={(event) => commitDate(event.target.value, "end_date", setEndError, true)}
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
      {filters.application_type.map((value) => (
        <ActiveFilterChip
          key={`type-${value}`}
          field="Candidature"
          value={ApplicationTypes.find((type) => type.value === value)?.label ?? value}
          onRemove={() =>
            onApply(
              patch(filters, {
                application_type: filters.application_type.filter((item) => item !== value),
              }),
            )
          }
        />
      ))}
      {filters.contract_type_code.map((value) => (
        <ActiveFilterChip
          key={`contract-${value}`}
          field="Contrat"
          value={referenceLabel(referentials.data.contract_types, value) ?? value}
          onRemove={() =>
            onApply(
              patch(filters, {
                contract_type_code: filters.contract_type_code.filter((item) => item !== value),
              }),
            )
          }
        />
      ))}
      {filters.professional_domain_id.map((value) => (
        <ActiveFilterChip
          key={`domain-${value}`}
          field="Domaine"
          value={referenceLabel(referentials.data.professional_domains, value) ?? value}
          onRemove={() =>
            onApply(
              patch(filters, {
                professional_domain_id: filters.professional_domain_id.filter(
                  (item) => item !== value,
                ),
              }),
            )
          }
        />
      ))}
      {filters.company_type_id.map((value) => (
        <ActiveFilterChip
          key={`company-type-${value}`}
          field="Type d'entreprise"
          value={referenceLabel(referentials.data.company_types, value) ?? value}
          onRemove={() =>
            onApply(
              patch(filters, {
                company_type_id: filters.company_type_id.filter((item) => item !== value),
              }),
            )
          }
        />
      ))}
      {filters.sector_id.map((value) => (
        <ActiveFilterChip
          key={`sector-${value}`}
          field="Secteur"
          value={
            referentials.data.sectors.find((sector) => sector.id === value)?.name ?? value
          }
          onRemove={() =>
            onApply(
              patch(filters, {
                sector_id: filters.sector_id.filter((item) => item !== value),
              }),
            )
          }
        />
      ))}
      {filters.company_size.map((value) => (
        <ActiveFilterChip
          key={`size-${value}`}
          field="Taille"
          value={CompanySizes.find((size) => size.value === value)?.label ?? value}
          onRemove={() =>
            onApply(
              patch(filters, {
                company_size: filters.company_size.filter((item) => item !== value),
              }),
            )
          }
        />
      ))}
      {filters.weekly_work_schedule.map((value) => (
        <ActiveFilterChip
          key={`schedule-${value}`}
          field="Durée"
          value={
            WeeklyWorkSchedules.find((schedule) => schedule.value === value)?.label ?? value
          }
          onRemove={() =>
            onApply(
              patch(filters, {
                weekly_work_schedule: filters.weekly_work_schedule.filter(
                  (item) => item !== value,
                ),
              }),
            )
          }
        />
      ))}
      {hours ? (
        <ActiveFilterChip
          field="Heures"
          value={hours}
          onRemove={() =>
            onApply(patch(filters, { min_weekly_hours: null, max_weekly_hours: null }))
          }
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
      {total !== null ? (
        <p className="tabular text-note font-semibold text-ink" aria-live="polite">
          {total} candidature{total === 1 ? "" : "s"}
        </p>
      ) : null}
    </FilterBar>
  );
}
