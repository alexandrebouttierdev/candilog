import {
  AI_TASKS,
  assignmentOf,
  getProvider,
  idProvider,
  isAiConfigured,
  useManagedOllamaViewModel,
  useSettingsViewModel,
} from "@/features/settings";
import type { AiTask } from "@/shared/types/generated/settings";

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

export interface AiIndicator {
  readonly label: string;
  readonly model: string | null;
  readonly locality: Locality;
  /** Résumé du routage (« 3 loc · 1 dist ») quand au moins une tâche a son propre modèle. */
  readonly routing: string | null;
  /** Qui fera la tâche, et où : « Ministral 3 · sur votre ordinateur ». */
  readonly taskDetail: (task: AiTask) => string;
}

/**
 * Indicateur d'IA : fournisseur principal, modèle et localité pour le pied de navigation ;
 * modèle et localité de chaque tâche (« Qui fait quoi ») pour la palette.
 */
export function useAiIndicator(): AiIndicator {
  const settings = useSettingsViewModel();
  const managed = useManagedOllamaViewModel(undefined, { enabled: true });
  const data = settings.data;
  const main = mainIndicator(settings.data?.llm, managed.status?.active_model?.display_name ?? null);
  if (!data) return { ...main, routing: null, taskDetail: () => main.label };

  const models = managed.status?.models ?? [];
  const assignments = AI_TASKS.map((task) => ({ task: task.value, ...assignmentOf(task.value, data, models) }));
  const routed = Object.keys(data.ai_routes).length > 0;
  const local = assignments.filter((item) => item.locality === "local").length;
  const remote = assignments.filter((item) => item.locality === "remote").length;

  return {
    ...main,
    routing: routed ? `${local} loc · ${remote} dist` : null,
    taskDetail: (task) => {
      const assignment = assignments.find((item) => item.task === task);
      if (!assignment || assignment.locality === "off") return "tâche désactivée";
      // Sans modèle attitré, la tâche suit le principal : s'il n'est pas prêt, elle non plus.
      if (assignment.isDefault && main.locality === "none") return "IA non configurée";
      return `${assignment.label} · ${assignment.locality === "local" ? "sur votre ordinateur" : "envoi distant"}`;
    },
  };
}

function mainIndicator(
  llm: NonNullable<ReturnType<typeof useSettingsViewModel>["data"]>["llm"] | undefined,
  activeLocalModel: string | null,
): { label: string; model: string | null; locality: Locality } {
  if (!llm) return { label: "IA", model: null, locality: "none" };

  const id = idProvider(llm.provider);
  if (id === "candilog_local") {
    return { label: "IA locale", model: activeLocalModel, locality: activeLocalModel ? "local" : "none" };
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
