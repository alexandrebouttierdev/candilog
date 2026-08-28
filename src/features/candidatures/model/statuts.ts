import type { StatutCandidature, TypeContrat } from "@/shared/types/generated/candidatures";
import type { Tone } from "@/shared/ui";

export type { StatutCandidature, TypeContrat };

/** Présentation d'un statut : libellé, tonalité, icône. */
export interface StatutMeta {
  readonly valeur: StatutCandidature;
  readonly label: string;
  readonly tone: Tone;
  readonly icon: string;
}

/**
 * Les quatre statuts, dans l'ordre des colonnes du Kanban.
 *
 * Les tonalités viennent des maquettes : vert pour l'avancement, ambre pour ce qui est à
 * traiter, rouge pour l'échec, neutre pour l'attente. La couleur ne porte jamais
 * l'information seule — chaque pastille affiche son libellé.
 */
export const STATUTS: readonly StatutMeta[] = [
  { valeur: "EN_ATTENTE", label: "En attente", tone: "neutral", icon: "hourglass_top" },
  { valeur: "RELANCEE", label: "Relancée", tone: "warning", icon: "send" },
  { valeur: "ENTRETIEN", label: "Entretien", tone: "success", icon: "event_available" },
  { valeur: "REFUS", label: "Refusée", tone: "danger", icon: "do_not_disturb_on" },
] as const;

/** Présentation d'un statut donné. */
export function statutMeta(valeur: StatutCandidature): StatutMeta {
  return STATUTS.find((statut) => statut.valeur === valeur) ?? STATUTS[0]!;
}

/**
 * Types de contrat, dans l'ordre du sélecteur.
 *
 * Les valeurs reprennent la casse exacte contrainte en base par la migration 005 : les
 * modifier romprait la lecture des données existantes.
 */
export const CONTRATS: readonly TypeContrat[] = [
  "CDI",
  "CDD",
  "Freelance",
  "Stage",
  "Alternance",
  "Interim",
  "Autre",
] as const;

/** Libellé affiché d'un type de contrat. */
export function contratLabel(contrat: TypeContrat): string {
  return contrat === "Interim" ? "Intérim" : contrat;
}
