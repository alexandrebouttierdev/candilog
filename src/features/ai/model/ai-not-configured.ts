/**
 * Levée lorsqu'une action IA est demandée sans fournisseur ni modèle prêts.
 *
 * La modale centrale prend le relais : les écrans doivent ignorer cette erreur
 * plutôt que d'afficher un bandeau redondant.
 */
export class AiNotConfiguredError extends Error {
  readonly code = "AI_NOT_CONFIGURED";

  constructor() {
    super("Configurez un fournisseur IA pour continuer.");
    this.name = "AiNotConfiguredError";
  }
}

export function isAiNotConfiguredError(error: unknown): error is AiNotConfiguredError {
  return error instanceof AiNotConfiguredError;
}
