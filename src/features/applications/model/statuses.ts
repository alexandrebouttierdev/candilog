import type { ApplicationStatus } from "@/shared/types/generated/applications";
import type { GlyphTone, Tone } from "@/shared/ui";
import type { IconName } from "@/shared/ui/icon-names";

export type { ApplicationStatus };

/** Présentation d'un statut : libellé, tonalité, icône. */
export interface StatusMeta {
  readonly value: ApplicationStatus;
  readonly label: string;
  readonly tone: Tone;
  readonly icon: IconName;
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
  { value: "EN_ATTENTE", label: "En attente", tone: "neutral", icon: "hourglass_top", glyph: "n" },
  { value: "RELANCEE", label: "Relancée", tone: "warning", icon: "send", glyph: "a" },
  { value: "ENTRETIEN", label: "Entretien", tone: "success", icon: "event_available", glyph: "g" },
  { value: "REFUS", label: "Refusée", tone: "danger", icon: "do_not_disturb_on", glyph: "c" },
] as const;

/** Présentation d'un statut donné. */
export function status_meta(value: ApplicationStatus): StatusMeta {
  return Statuses.find((status) => status.value === value) ?? Statuses[0]!;
}
