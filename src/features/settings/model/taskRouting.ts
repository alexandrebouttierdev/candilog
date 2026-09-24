import type { AiTask, ProviderKind, Settings, TaskRoute } from "@/shared/types/generated/settings";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { getProvider, idProvider } from "./providers";

/** Les cinq tâches routables, dans l'ordre de l'écran (`AI_TASK_ROUTING.md` §2). */
export const AI_TASKS: ReadonlyArray<{ value: AiTask; label: string }> = [
  { value: "generate_resume", label: "Générer un CV ciblé" },
  { value: "write_letter", label: "Rédiger une lettre" },
  { value: "analyze_resume", label: "Analyser un CV" },
  { value: "extract_offer", label: "Extraire une offre d'emploi" },
  { value: "import_resume", label: "Lire un CV importé" },
];

/** Assignation lisible d'une tâche. */
export interface Assignment {
  readonly label: string;
  /** `local` : rien ne quitte l'ordinateur ; `remote` : envoi au fournisseur ; `off` : désactivée. */
  readonly locality: "local" | "remote" | "off";
  readonly isDefault: boolean;
}

export function isLocal(provider: ProviderKind): boolean {
  const id = idProvider(provider);
  return id === "candilog_local" || id === "ollama";
}

function routeLabel(route: TaskRoute, models: readonly ManagedModelStatus[]): string {
  if (idProvider(route.provider) === "candilog_local") {
    return models.find((model) => model.definition.ollama_tag === route.model)?.definition.display_name ?? route.model;
  }
  return `${getProvider(route.provider).label} · ${route.model}`;
}

/** Libellé du fournisseur principal, suivi par les tâches sans route. */
export function mainLabel(settings: Settings, models: readonly ManagedModelStatus[]): string {
  if (idProvider(settings.llm.provider) === "candilog_local") {
    return models.find((model) => model.active)?.definition.display_name ?? "IA locale";
  }
  const provider = getProvider(settings.llm.provider).label;
  return settings.llm.model ? `${provider} · ${settings.llm.model}` : provider;
}

export function assignmentOf(task: AiTask, settings: Settings, models: readonly ManagedModelStatus[]): Assignment {
  if (!(task in settings.ai_routes)) {
    return {
      label: mainLabel(settings, models),
      locality: isLocal(settings.llm.provider) ? "local" : "remote",
      isDefault: true,
    };
  }
  const route = settings.ai_routes[task];
  if (!route) return { label: "Désactivée", locality: "off", isDefault: false };
  return { label: routeLabel(route, models), locality: isLocal(route.provider) ? "local" : "remote", isDefault: false };
}
