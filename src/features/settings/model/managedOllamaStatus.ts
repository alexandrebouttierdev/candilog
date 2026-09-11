import type { ManagedOllamaStatus, ManagedRuntimeState } from "@/shared/types/generated/ai";
import type { AiStatus } from "./aiStatus";

export function isManagedOllamaBusy(state: ManagedRuntimeState): boolean {
  return (
    state === "downloading" ||
    state === "installing" ||
    state === "starting" ||
    state === "updating" ||
    state === "stopping"
  );
}

/** État du bandeau IA pour l'Ollama géré Candilog (même vocabulaire que `aiStatus`). */
export function managedOllamaStatus(
  status: ManagedOllamaStatus | null,
  error: string | null,
): AiStatus {
  if (!status) {
    return { label: "Chargement…", tone: "neutral", hint: null };
  }
  if (isManagedOllamaBusy(status.runtime_state)) {
    return { label: "Installation en cours", tone: "neutral", hint: null };
  }
  if (status.runtime_state === "error" || error || status.last_error) {
    return { label: "Erreur", tone: "danger", hint: status.last_error ?? error };
  }
  if (!status.active_model) {
    return {
      label: "Non configuré",
      tone: "warning",
      hint: "Téléchargez un modèle optimisé pour votre ordinateur.",
    };
  }
  if (status.runtime_state === "ready") {
    return { label: "Prêt", tone: "success", hint: null };
  }
  return {
    label: "Configuré",
    tone: "accent",
    hint: "Testez l'IA pour confirmer que le modèle local répond.",
  };
}
