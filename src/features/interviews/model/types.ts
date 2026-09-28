import type { InterviewType } from "@/shared/types/generated/interviews";

export type { InterviewType };

/**
 * Formats d'entretien, dans l'ordre du sélecteur.
 *
 * Les valeurs reprennent la casse et les accents contraints en base par la migration 005 :
 * les modifier romprait la lecture des lignes existantes.
 */
export const INTERVIEW_TYPES: readonly InterviewType[] = [
  "Présentiel",
  "Visio",
  "Téléphonique",
  "Technique",
  "RH",
  "Autre",
] as const;
