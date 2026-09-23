import {
  getProvider,
  idProvider,
  isAiConfigured,
  useManagedOllamaViewModel,
  useSettingsViewModel,
} from "@/features/settings";

/** Localité d'un point de terminaison : la puce suit l'adresse, pas le fournisseur. */
export type Locality = "local" | "remote" | "none";

/**
 * Une adresse de cette machine ? `localhost`, `127.0.0.1`, `[::1]`, `*.local` sont locales ;
 * toute autre adresse — y compris une IP de réseau privé — est distante, car les données
 * quittent la machine (`reference_design/DECISIONS.md` D13, volontairement prudent).
 */
export function isLocalEndpoint(endpoint: string | null | undefined): boolean {
  if (!endpoint) return false;
  try {
    const host = new URL(endpoint).hostname.toLowerCase();
    return (
      host === "localhost" ||
      host === "127.0.0.1" ||
      host === "[::1]" ||
      host === "::1" ||
      host.endsWith(".local")
    );
  } catch {
    return false;
  }
}

/**
 * Indicateur d'IA du pied de navigation : fournisseur actif, modèle et localité.
 *
 * Transitoire : tant que le routage par tâche (« Qui fait quoi ») n'existe pas, Candilog a
 * un fournisseur actif unique, et c'est lui que l'indicateur décrit.
 */
export function useAiIndicator(): { label: string; model: string | null; locality: Locality } {
  const settings = useSettingsViewModel();
  const managed = useManagedOllamaViewModel(undefined, { enabled: true });
  const llm = settings.data?.llm;
  if (!llm) return { label: "IA", model: null, locality: "none" };

  const id = idProvider(llm.provider);
  if (id === "candilog_local") {
    const model = managed.status?.active_model?.display_name ?? null;
    return { label: "IA locale", model, locality: model ? "local" : "none" };
  }
  if (!isAiConfigured(llm)) return { label: "IA non configurée", model: null, locality: "none" };
  // Ollama sans adresse explicite tourne sur son port local par défaut.
  const local = id === "ollama" && !llm.endpoint ? true : isLocalEndpoint(llm.endpoint);
  return {
    label: getProvider(llm.provider).label,
    model: llm.model.trim() || null,
    locality: local ? "local" : "remote",
  };
}
