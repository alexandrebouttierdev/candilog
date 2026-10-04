import type { AiTask, Settings, TaskRoute } from "@/shared/types/generated/settings";
import type { ManagedModelStatus } from "@/shared/types/generated/ai";
import { defaultEndpoint, getProvider, idProvider } from "./providers";
import type { ProviderOption } from "./providers";

/** Les cinq tâches routables, dans l'ordre de l'écran (`docs/AI.md`, « Routage par tâche »). */
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

/**
 * Une adresse de cette machine ? `localhost`, `127.0.0.1`, `[::1]`, `*.local` sont locales ;
 * toute autre adresse — y compris une IP de réseau privé — est distante, car les données
 * quittent la machine (décision D13, `docs/AI.md` ; volontairement prudent).
 */
export function isLocalEndpoint(endpoint: string | null | undefined): boolean {
  if (!endpoint) return false;
  try {
    const host = new URL(endpoint).hostname.toLowerCase();
    return host === "localhost" || host === "127.0.0.1" || host === "[::1]" || host === "::1" || host.endsWith(".local");
  } catch {
    return false;
  }
}

/** Qui reçoit les données : l'éditeur du service, ou l'hôte pour un serveur que l'on désigne. */
const RECIPIENTS: Partial<Record<ProviderOption["id"], string>> = {
  claude: "Anthropic",
  openai: "OpenAI",
  gemini: "Google",
  mistral: "Mistral AI",
  deepseek: "DeepSeek",
};

/** Où part une tâche, et si ses données quittent l'ordinateur. */
export interface TaskDestination {
  readonly providerId: ProviderOption["id"];
  readonly recipient: string;
  readonly remote: boolean;
}

/**
 * Destination réelle d'une tâche : sa route, sinon le fournisseur principal ; `null` si elle
 * est désactivée. La localité suit l'adresse (D13), pas le nom du fournisseur : un Ollama
 * sur une autre machine est un envoi distant.
 */
export function taskDestination(task: AiTask, settings: Settings): TaskDestination | null {
  const routed = task in settings.ai_routes;
  const route = routed ? settings.ai_routes[task] : undefined;
  if (routed && !route) return null;
  const provider = route ? route.provider : settings.llm.provider;
  const providerId = idProvider(provider);
  if (providerId === "candilog_local") return { providerId, recipient: "votre ordinateur", remote: false };
  const endpoint = (route ? settings.llm_presets[providerId]?.endpoint : settings.llm.endpoint) ?? defaultEndpoint(providerId);
  let host = endpoint ?? "";
  try {
    host = endpoint ? new URL(endpoint).hostname : host;
  } catch {
    // Adresse illisible : on la montre telle quelle, et elle compte comme distante.
  }
  return {
    providerId,
    recipient: RECIPIENTS[providerId] ?? host,
    remote: !isLocalEndpoint(endpoint),
  };
}

/** Destination à faire confirmer avant l'envoi (D4), ou `null` si rien n'est à demander. */
export function remoteSendToConfirm(task: AiTask, settings: Settings): TaskDestination | null {
  const destination = taskDestination(task, settings);
  if (!destination?.remote) return null;
  return settings.remote_send_consents.includes(destination.providerId) ? null : destination;
}

/** Nombre de tâches qu'un fournisseur distant recevrait : « Tâches concernées — 1 sur 5 ». */
export function tasksSentTo(providerId: ProviderOption["id"], settings: Settings): number {
  return AI_TASKS.filter((task) => {
    const destination = taskDestination(task.value, settings);
    return destination?.remote === true && destination.providerId === providerId;
  }).length;
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
  const locality = taskDestination(task, settings)?.remote ? "remote" : "local";
  if (!(task in settings.ai_routes)) {
    return { label: mainLabel(settings, models), locality, isDefault: true };
  }
  const route = settings.ai_routes[task];
  if (!route) return { label: "Désactivée", locality: "off", isDefault: false };
  return { label: routeLabel(route, models), locality, isDefault: false };
}
