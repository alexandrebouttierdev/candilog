import type { ApplicationStatus } from "@/shared/types/generated/applications";
import type { GlyphTone, Tone } from "@/shared/ui";

export type { ApplicationStatus };

/** Présentation d'un statut : libellé, tonalité, glyphe. */
export interface StatusMeta {
  readonly value: ApplicationStatus;
  readonly label: string;
  readonly tone: Tone;
  /** Remplissage du glyphe de statut v2 : vide, demi, trois quarts, plein. */
  readonly glyph: GlyphTone;
}

/**
 * Les quatre statuts, dans l'ordre des colonnes du Kanban.
 *
 * Les tonalités viennent des maquettes : vert pour l'avancement, ambre pour ce qui est à
 * traiter, rouge pour l'échec, neutre pour l'attente. La couleur ne porte jamais
 * l'information seule — chaque pastille affiche son libellé.
 */
export const Statuses: readonly StatusMeta[] = [
  { value: "EN_ATTENTE", label: "En attente", tone: "neutral", glyph: "n" },
  { value: "RELANCEE", label: "Relancée", tone: "warning", glyph: "a" },
  { value: "ENTRETIEN", label: "Entretien", tone: "success", glyph: "g" },
  { value: "REFUS", label: "Refusée", tone: "danger", glyph: "c" },
] as const;

/** Présentation d'un statut donné. */
export function status_meta(value: ApplicationStatus): StatusMeta {
  return Statuses.find((status) => status.value === value) ?? Statuses[0]!;
}
