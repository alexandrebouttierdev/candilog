import type { FilterField } from "@/shared/types/generated/applications";
import type { ApplicationFilterValues } from "./schemas/application-filter.schema";

/**
 * Champs du menu « + Filtre » (`INTERACTIONS.md` §3.2) : un premier niveau choisit le champ,
 * le second sa valeur. Tous les critères du filtre backend y figurent — les quatre de la
 * maquette (statut, contrat, entreprise, envoyée) et les critères fins de la v1, qu'il ne
 * faut pas perdre (domaine, type et taille d'entreprise, secteur, régime, heures, poste,
 * ville, période).
 */
export type FilterKey =
  | "status"
  | "channel"
  | "contract"
  | "company"
  | "sent"
  | "application_type"
  | "domain"
  | "company_type"
  | "sector"
  | "size"
  | "schedule"
  | "hours"
  | "job_title"
  | "city";

/** Critères à valeurs multiples, et la clé du filtre qui les porte. */
export const LIST_KEYS = {
  status: "status",
  channel: "channel",
  contract: "contract_type_code",
  application_type: "application_type",
  domain: "professional_domain_id",
  company_type: "company_type_id",
  sector: "sector_id",
  size: "company_size",
  schedule: "weekly_work_schedule",
} as const satisfies Partial<Record<FilterKey, keyof ApplicationFilterValues>>;

export type ListKey = keyof typeof LIST_KEYS;

/** Critère inversable côté backend, par champ du menu. Les bornes ne s'inversent pas. */
export const NEGATABLE: Partial<Record<FilterKey, FilterField>> = {
  status: "status",
  channel: "channel",
  contract: "contract_type",
  application_type: "application_type",
  domain: "professional_domain",
  company_type: "company_type",
  sector: "sector",
  size: "company_size",
  schedule: "weekly_work_schedule",
  company: "company",
  job_title: "job_title",
  city: "city",
};

export const FIELD_LABELS: Record<FilterKey, string> = {
  status: "Statut",
  channel: "Canal",
  contract: "Contrat",
  company: "Entreprise",
  sent: "Envoyée",
  application_type: "Démarche",
  domain: "Domaine",
  company_type: "Type d'entreprise",
  sector: "Secteur",
  size: "Taille",
  schedule: "Régime",
  hours: "Heures",
  job_title: "Poste",
  city: "Ville",
};

/** Ordre du premier niveau : les champs de la maquette d'abord, les critères fins ensuite. */
export const FIELD_ORDER: readonly FilterKey[] = [
  "status", "contract", "company", "sent", "channel", "job_title", "city",
  "application_type", "domain", "company_type", "sector", "size", "schedule", "hours",
];

export function isListKey(key: FilterKey): key is ListKey {
  return key in LIST_KEYS;
}

function listOf(filters: ApplicationFilterValues, key: ListKey): readonly string[] {
  return filters[LIST_KEYS[key]];
}

/** Coche ou décoche une valeur d'un critère à valeurs multiples. */
export function toggleValue(
  filters: ApplicationFilterValues,
  key: ListKey,
  value: string,
): ApplicationFilterValues {
  const current = listOf(filters, key);
  const next = current.includes(value) ? current.filter((item) => item !== value) : [...current, value];
  const cleared = next.length === 0 ? withoutExclusion(filters, key) : filters;
  return { ...cleared, [LIST_KEYS[key]]: next };
}

function withoutExclusion(filters: ApplicationFilterValues, key: FilterKey): ApplicationFilterValues {
  const field = NEGATABLE[key];
  return field ? { ...filters, excluded: filters.excluded.filter((item) => item !== field) } : filters;
}

/** Retire entièrement un critère (la croix de sa puce). */
export function removeField(filters: ApplicationFilterValues, key: FilterKey): ApplicationFilterValues {
  const base = withoutExclusion(filters, key);
  if (isListKey(key)) return { ...base, [LIST_KEYS[key]]: [] };
  switch (key) {
    case "company":
      return { ...base, company_id: null };
    case "sent":
      return { ...base, start_date: null, end_date: null };
    case "hours":
      return { ...base, min_weekly_hours: null, max_weekly_hours: null };
    case "job_title":
      return { ...base, job_title: "" };
    case "city":
      return { ...base, city: "" };
  }
}

/** Inverse la condition d'un critère (clic sur sa puce) ; sans effet sur une borne. */
export function toggleExcluded(filters: ApplicationFilterValues, key: FilterKey): ApplicationFilterValues {
  const field = NEGATABLE[key];
  if (!field) return filters;
  const excluded = filters.excluded.includes(field)
    ? filters.excluded.filter((item) => item !== field)
    : [...filters.excluded, field];
  return { ...filters, excluded };
}

/** Le critère est-il posé ? */
export function isActive(filters: ApplicationFilterValues, key: FilterKey): boolean {
  if (isListKey(key)) return listOf(filters, key).length > 0;
  switch (key) {
    case "company":
      return filters.company_id !== null;
    case "sent":
      return filters.start_date !== null || filters.end_date !== null;
    case "hours":
      return filters.min_weekly_hours !== null || filters.max_weekly_hours !== null;
    case "job_title":
      return filters.job_title !== "";
    case "city":
      return filters.city !== "";
  }
}

/** Puce d'un critère actif : « Contrat est CDI, CDD », « Poste ne contient pas stage ». */
export interface FilterChip {
  readonly key: FilterKey;
  readonly field: string;
  readonly op: string;
  readonly value: string;
  /** Un clic inverse la condition. */
  readonly negatable: boolean;
}

/** `AAAA-MM-JJ` → `JJ-MM-AAAA`, la notation des dates saisies. */
function displayDate(iso: string): string {
  return `${iso.slice(8, 10)}-${iso.slice(5, 7)}-${iso.slice(0, 4)}`;
}

function formatNumber(hours: number): string {
  return String(hours).replace(".", ",");
}

/**
 * Puces des critères actifs, dans l'ordre du menu. `label` traduit une valeur codée (code
 * de contrat, identifiant de secteur, d'entreprise) en libellé lisible.
 */
export function chipsOf(
  filters: ApplicationFilterValues,
  label: (key: FilterKey, value: string) => string,
): FilterChip[] {
  return FIELD_ORDER.filter((key) => isActive(filters, key)).map((key) => {
    const field = NEGATABLE[key];
    const negated = field !== undefined && filters.excluded.includes(field);
    const base = { key, field: FIELD_LABELS[key], negatable: field !== undefined };

    if (isListKey(key)) {
      const values = listOf(filters, key).map((value) => label(key, value));
      return { ...base, op: negated ? "n'est pas" : "est", value: values.join(", ") };
    }
    switch (key) {
      case "company":
        return { ...base, op: negated ? "n'est pas" : "est", value: label(key, filters.company_id ?? "") };
      case "job_title":
        return { ...base, op: negated ? "ne contient pas" : "contient", value: filters.job_title };
      case "city":
        return { ...base, op: negated ? "n'est pas" : "est", value: filters.city };
      case "hours": {
        const { min_weekly_hours: min, max_weekly_hours: max } = filters;
        if (min !== null && max !== null) return { ...base, op: "entre", value: `${formatNumber(min)} et ${formatNumber(max)} h` };
        if (min !== null) return { ...base, op: "≥", value: `${formatNumber(min)} h` };
        return { ...base, op: "≤", value: `${formatNumber(max ?? 0)} h` };
      }
      case "sent": {
        const { start_date: start, end_date: end } = filters;
        if (start && end) return { ...base, op: "entre le", value: `${displayDate(start)} et le ${displayDate(end)}` };
        if (start) return { ...base, op: "depuis le", value: displayDate(start) };
        return { ...base, op: "jusqu'au", value: displayDate(end ?? "") };
      }
    }
  });
}

/** Préréglages de « Envoyée » : il y a plus de N jours (borne haute de la date d'envoi). */
export const SENT_PRESETS = [7, 14, 30] as const;

/** Date `AAAA-MM-JJ` située `days` jours avant `today` (locale). */
export function daysAgo(days: number, today = new Date()): string {
  const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() - days);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}
