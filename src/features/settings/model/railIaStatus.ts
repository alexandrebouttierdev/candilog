import type { Tone } from "@/shared/ui";

/** Entrées pour la pastille du rail (état connu uniquement, pas de ping). */
export interface RailIaStatusInput {
  readonly configured: boolean;
  readonly busy: boolean;
  readonly localError: boolean;
  readonly connectionError: boolean;
  readonly operationError: boolean;
}

export interface RailIaStatus {
  readonly tone: Tone;
  readonly label: string;
}

/**
 * Priorité : occupation → erreur connue → non configuré → disponible.
 */
export function railIaStatus(input: RailIaStatusInput): RailIaStatus {
  if (input.busy) {
    return { tone: "warning", label: "En cours…" };
  }
  if (input.localError || input.connectionError || input.operationError) {
    return { tone: "danger", label: "Erreur" };
  }
  if (!input.configured) {
    return { tone: "neutral", label: "Non configuré" };
  }
  return { tone: "success", label: "Disponible" };
}
