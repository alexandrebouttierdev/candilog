import type { CompanySize } from "@/shared/types/generated/companies";
import type {
  ApplicationType,
  WeeklyWorkSchedule,
} from "@/shared/types/generated/applications";

export type { ApplicationType, CompanySize, WeeklyWorkSchedule };

/** Option d'un sélecteur : valeur persistée et libellé affiché. */
export interface EnumOption<T extends string> {
  readonly value: T;
  readonly label: string;
}

/**
 * Libellés des trois enums fermés du modèle.
 *
 * Contrairement aux quatre référentiels, ces jeux de valeurs sont contraints par un `CHECK`
 * du schéma et par un enum Rust : les étendre demande une modification du code des deux
 * côtés. Leur libellé vit donc ici, et non en base — il n'y a rien à charger.
 */
export const CompanySizes: readonly EnumOption<CompanySize>[] = [
  { value: "MICRO", label: "Micro-entreprise" },
  { value: "TPE", label: "TPE" },
  { value: "PME", label: "PME" },
  { value: "ETI", label: "ETI" },
  { value: "LARGE", label: "Grande entreprise / Grand groupe" },
  { value: "UNKNOWN", label: "Non renseignée" },
] as const;

export const ApplicationTypes: readonly EnumOption<ApplicationType>[] = [
  { value: "OFFRE", label: "Offre d'emploi" },
  { value: "SPONTANEE", label: "Candidature spontanée" },
] as const;

export const WeeklyWorkSchedules: readonly EnumOption<WeeklyWorkSchedule>[] = [
  { value: "FULL_TIME", label: "Temps plein" },
  { value: "PART_TIME", label: "Temps partiel" },
  { value: "UNSPECIFIED", label: "Non renseignée" },
] as const;

/** Libellé d'une valeur, ou la valeur elle-même si elle sort du jeu attendu. */
function label<T extends string>(options: readonly EnumOption<T>[], value: T): string {
  return options.find((option) => option.value === value)?.label ?? value;
}

/** Libellé d'une taille d'entreprise. */
export function companySizeLabel(value: CompanySize): string {
  return label(CompanySizes, value);
}

/** Libellé d'une nature de candidature. */
export function applicationTypeLabel(value: ApplicationType): string {
  return label(ApplicationTypes, value);
}

/** Libellé d'un régime horaire, sans son volume. */
export function weeklyScheduleLabel(value: WeeklyWorkSchedule): string {
  return label(WeeklyWorkSchedules, value);
}

/**
 * Durée hebdomadaire complète : régime, puis volume horaire s'il est renseigné.
 *
 * « Temps plein · 35 h/semaine », « Temps partiel · 17,5 h/semaine ». Les décimales sont
 * écrites à la française — un « 17.5 » au milieu d'une interface en français se lit mal.
 * Un régime non renseigné n'a jamais de volume à afficher : les deux se contrediraient.
 */
export function weeklyDurationLabel(
  schedule: WeeklyWorkSchedule,
  hours: number | null,
): string {
  const base = weeklyScheduleLabel(schedule);
  if (hours === null || schedule === "UNSPECIFIED") return base;
  return `${base} · ${formatHours(hours)} h/semaine`;
}

/** Volume horaire au format français, sans décimale inutile. */
export function formatHours(hours: number): string {
  return hours.toLocaleString("fr-FR", { maximumFractionDigits: 2 });
}
