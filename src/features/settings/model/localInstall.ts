import type {
  MachineFit,
  ManagedModelCategory,
  ManagedModelDefinition,
  ManagedOllamaDownloadProgress,
} from "@/shared/types/generated/ai";

/** Taille lisible d'un téléchargement : « 42 Mo », « 2,1 Go ». */
export function formatBytes(bytes: number): string {
  if (bytes < 1_000_000_000) {
    return `${(bytes / 1_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 0 })} Mo`;
  }
  return `${(bytes / 1_000_000_000).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} Go`;
}

export const CATEGORY_LABELS: Record<ManagedModelCategory, string> = {
  ultra_light: "Très léger",
  light: "Léger",
  balanced: "Équilibré",
  powerful: "Puissant",
  max_quality: "Qualité maximale",
};

export const MACHINE_FIT_LABELS: Record<MachineFit, string> = {
  recommended: "Recommandé",
  compatible: "Compatible",
  may_be_slow: "Peut être lent",
  insufficient_memory: "Mémoire insuffisante",
};

/**
 * Jauges d'une carte de modèle (`screens/14`), de 1 à 3. Vitesse et qualité traduisent la
 * catégorie que le catalogue attribue au modèle ; la mémoire suit la RAM qu'il recommande.
 * Aucune mesure n'est inventée : c'est la classification du catalogue, dessinée.
 */
export function modelMeters(definition: ManagedModelDefinition): { speed: number; quality: number; memory: number } {
  const byCategory: Record<ManagedModelCategory, { speed: number; quality: number }> = {
    ultra_light: { speed: 3, quality: 1 },
    light: { speed: 3, quality: 2 },
    balanced: { speed: 2, quality: 2 },
    powerful: { speed: 1, quality: 3 },
    max_quality: { speed: 1, quality: 3 },
  };
  const ram = definition.recommended_ram_gb;
  return { ...byCategory[definition.category], memory: ram <= 4 ? 1 : ram <= 8 ? 2 : 3 };
}

/** Phases de l'installation guidée (`INTERACTIONS.md` §4.3) : choix, déroulé, fin. */
export type InstallPhase = "pick" | "run" | "done";

export type InstallStepKey = "engine" | "model" | "verify";
export type InstallStepState = "pending" | "running" | "done";

/**
 * Étape en cours d'après le dernier événement de progression : le moteur (téléchargement,
 * extraction, démarrage) précède toujours le modèle. Sans événement, l'installation
 * commence par le moteur.
 */
export function currentStep(progress: ManagedOllamaDownloadProgress | null): InstallStepKey {
  return progress?.kind === "model" ? "model" : "engine";
}

const STEP_ORDER: readonly InstallStepKey[] = ["engine", "model", "verify"];

/** La plus avancée de deux étapes : le déroulé ne revient jamais en arrière. */
export function laterStep(previous: InstallStepKey | null, next: InstallStepKey): InstallStepKey {
  return previous !== null && STEP_ORDER.indexOf(previous) > STEP_ORDER.indexOf(next) ? previous : next;
}

export function stepState(step: InstallStepKey, current: InstallStepKey | null): InstallStepState {
  if (current === null) return "pending";
  const gap = STEP_ORDER.indexOf(step) - STEP_ORDER.indexOf(current);
  return gap < 0 ? "done" : gap === 0 ? "running" : "pending";
}

/**
 * Débit et temps restant d'un téléchargement, calculés sur les octets reçus depuis le début
 * de l'étape. `null` tant que la mesure n'a pas de sens (rien reçu, total inconnu).
 */
export function throughput(
  downloaded: number,
  total: number,
  elapsedMs: number,
): { bytesPerSecond: number; remainingSeconds: number } | null {
  if (downloaded <= 0 || total <= 0 || elapsedMs < 500) return null;
  const bytesPerSecond = downloaded / (elapsedMs / 1000);
  return { bytesPerSecond, remainingSeconds: Math.max(0, Math.round((total - downloaded) / bytesPerSecond)) };
}

/** « 12 s », « 3 min 05 ». */
export function formatRemaining(seconds: number): string {
  if (seconds < 60) return `${seconds} s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes} min ${String(seconds % 60).padStart(2, "0")}`;
}
