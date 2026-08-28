import type { InterviewType } from "@/shared/types/generated/interviews";

export type { InterviewType };

/**
 * Formats d'entretien, dans l'ordre du sélecteur.
 *
 * Les valeurs reprennent la casse et les accents contraints en base par la migration 005 :
 * les modifier romprait la lecture des lignes existantes.
 */
export const TYPES_INTERVIEW: readonly InterviewType[] = [
  "Présentiel",
  "Visio",
  "Téléphonique",
  "Technique",
  "RH",
  "Autre",
] as const;

/** Icône associée à un format d'entretien. */
export function interviewIcon(type: InterviewType): string {
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
