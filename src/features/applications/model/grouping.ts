import type { ApplicationFilter, ApplicationGrouping } from "@/shared/types/generated/applications";

/** Regroupement de la liste : le statut, ou un regroupement calculé par SQLite. */
export type ListGrouping = "status" | ApplicationGrouping;

/** Ordre du cycle « Grouper : statut ▾ » (`INTERACTIONS.md` §3.2). */
export const GROUPINGS: ReadonlyArray<{ value: ListGrouping; label: string }> = [
  { value: "status", label: "statut" },
  { value: "company", label: "entreprise" },
  { value: "contract", label: "contrat" },
];

/** Groupes ouverts d'emblée ; les suivants se déplient à la demande, et se chargent alors. */
export const OPEN_GROUPS = 8;

export function nextGrouping(current: ListGrouping): ListGrouping {
  const index = GROUPINGS.findIndex((grouping) => grouping.value === current);
  return GROUPINGS[(index + 1) % GROUPINGS.length]?.value ?? "status";
}

export function groupingLabel(grouping: ListGrouping): string {
  return GROUPINGS.find((entry) => entry.value === grouping)?.label ?? "statut";
}

/**
 * Filtre d'un groupe : le filtre de l'écran restreint à la clé du groupe. Une inversion du
 * même champ (« Contrat n'est pas CDI ») est déjà résolue par la liste des groupes : elle
 * ne doit pas s'appliquer à la clé elle-même.
 */
export function groupFilter(filter: ApplicationFilter, by: ApplicationGrouping, key: string): ApplicationFilter {
  if (by === "company") {
    return { ...filter, company_id: key, excluded: filter.excluded.filter((field) => field !== "company") };
  }
  return { ...filter, contract_type_code: [key], excluded: filter.excluded.filter((field) => field !== "contract_type") };
}
