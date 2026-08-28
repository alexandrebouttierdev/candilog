import type { TypeEntretien } from "@/shared/types/generated/entretiens";

export type { TypeEntretien };

/**
 * Formats d'entretien, dans l'ordre du sélecteur.
 *
 * Les valeurs reprennent la casse et les accents contraints en base par la migration 005 :
 * les modifier romprait la lecture des lignes existantes.
 */
export const TYPES_ENTRETIEN: readonly TypeEntretien[] = [
  "Présentiel",
  "Visio",
  "Téléphonique",
  "Technique",
  "RH",
  "Autre",
] as const;

/** Icône associée à un format d'entretien. */
export function iconeEntretien(type: TypeEntretien): string {
  switch (type) {
    case "Visio":
      return "videocam";
    case "Téléphonique":
      return "call";
    case "Technique":
      return "code";
    case "RH":
      return "groups";
    case "Présentiel":
      return "location_on";
    default:
      return "event";
  }
}
