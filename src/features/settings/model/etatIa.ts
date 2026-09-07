import type { LlmForm } from "@/shared/types/generated/settings";
import type { Tone } from "@/shared/ui";
import { idProvider } from "./providers";

/** Résultat du dernier test de connexion demandé par l'utilisateur. */
export type TestConnexion = "idle" | "pending" | "ok" | "error";

/** État affiché par l'écran Intelligence artificielle. */
export interface EtatIa {
  readonly label: string;
  readonly tone: Tone;
  /** Explication courte, affichée sous l'état ; `null` quand l'état se suffit. */
  readonly hint: string | null;
}

/**
 * Traduit la configuration et le dernier test en un état unique.
 *
 * Un seul état à l'écran, jamais deux pastilles concurrentes : le résultat d'un test récent
 * prime sur la configuration, qui prime sur l'absence de configuration. Sans cette fonction,
 * l'écran ne disait rien tant que personne n'avait cliqué sur « Tester ».
 */
export function etatIa(llm: LlmForm, test: TestConnexion): EtatIa {
  if (test === "pending") {
    return { label: "Connexion en cours", tone: "neutral", hint: null };
  }
  if (test === "error") {
    return { label: "Erreur", tone: "danger", hint: null };
  }
  if (test === "ok") {
    return { label: "Disponible", tone: "success", hint: null };
  }

  const manques = manquants(llm);
  if (manques.length > 0) {
    return {
      label: "Non configuré",
      tone: "warning",
      hint: `Renseignez ${manques.join(" et ")} pour utiliser l'assistance.`,
    };
  }
  return {
    label: "Configuré",
    tone: "accent",
    hint: "Testez la connexion pour confirmer que le fournisseur répond.",
  };
}

/** Indique si un fournisseur et un modèle sont prêts à lancer une action IA. */
export function iaEstConfiguree(llm: LlmForm): boolean {
  return manquants(llm).length === 0;
}

/** Champs indispensables encore vides, dans l'ordre où l'écran les présente. */
export function manquants(llm: LlmForm): string[] {
  const id = idProvider(llm.provider);
  // Mistral Local ne se décrit pas par `llm.model` : son artefact est désigné par
  // `local_ai.active_model_id`, et l'écran affiche MistralLocalPanel au lieu d'un champ
  // « Modèle ». Rien à réclamer ici — l'absence d'artefact installé est signalée par
  // l'écran dédié, et refusée côté Rust par `LocalAiService::provider`.
  if (id === "mistral_local") return [];
  const manques: string[] = [];
  if (llm.model.trim().length === 0) manques.push("le modèle");
  // Ollama tourne en local et fonctionne sans clé ; un endpoint compatible OpenAI peut
  // également être configuré sans secret. (Mistral Local a déjà rendu la main plus haut.)
  if (id !== "ollama" && id !== "custom" && !llm.api_key_configured) {
    manques.push("la clé API");
  }
  if (id === "custom" && (llm.endpoint ?? "").trim().length === 0) manques.push("l'endpoint");
  return manques;
}
