import type {
  LocalAiState,
  LocalModelDefinition,
} from "@/shared/types/generated/ai";
import type { EtatIa } from "./etatIa";

export function isLocalAiBusy(state: LocalAiState): boolean {
  return (
    state === "downloading" ||
    state === "verifying" ||
    state === "installing" ||
    state === "benchmarking"
  );
}

/** État du bandeau IA pour le fournisseur local (même vocabulaire que `etatIa`). */
export function etatLocalIa(
  state: LocalAiState,
  active: LocalModelDefinition | null,
  testResult: string | null,
  error: string | null,
): EtatIa {
  if (isLocalAiBusy(state)) {
    return { label: "Installation en cours", tone: "neutral", hint: null };
  }
  if (state === "detecting_hardware") {
    return { label: "Analyse…", tone: "neutral", hint: null };
  }
  if (state === "error" || error) {
    return { label: "Erreur", tone: "danger", hint: null };
  }
  if (!active) {
    return {
      label: "Non configuré",
      tone: "warning",
      hint: "Aucun compte ni clé API requis. Installez un profil pour utiliser l'assistance sur cet appareil.",
    };
  }
  if (testResult) {
    return { label: "Disponible", tone: "success", hint: null };
  }
  return {
    label: "Configuré",
    tone: "accent",
    hint: "Testez l'IA pour confirmer que le modèle local répond.",
  };
}
