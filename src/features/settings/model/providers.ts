import type {
  AnalysisMode,
  LlmForm,
  LlmProviderPresetForm,
  ProviderKind,
} from "@/shared/types/generated/settings";

export interface ProviderOption {
  readonly id:
    | "candilog_local"
    | "ollama"
    | "claude"
    | "openai"
    | "gemini"
    | "mistral"
    | "deepseek"
    | "custom";
  readonly label: string;
  readonly hint: string;
  readonly recommended?: boolean;
}

/** Grille des fournisseurs, jamais un menu déroulant. */
export const PROVIDERS: readonly ProviderOption[] = [
  {
    id: "candilog_local",
    label: "IA locale Candilog",
    hint: "Modèles gérés localement",
    recommended: true,
  },
  { id: "mistral", label: "Mistral", hint: "Europe" },
  { id: "openai", label: "OpenAI", hint: "GPT" },
  { id: "gemini", label: "Gemini", hint: "Google" },
  { id: "claude", label: "Claude", hint: "Anthropic" },
  { id: "deepseek", label: "DeepSeek", hint: "API" },
  {
    id: "custom",
    label: "Personnalisé",
    hint: "Compatible OpenAI, Ollama local, LM Studio, etc.",
  },
];

/** Fournisseur historique — conservé pour les bases déjà configurées, absent de la grille. */
export const OLLAMA_PROVIDER: ProviderOption = {
  id: "ollama",
  label: "Ollama",
  hint: "Votre installation ou Ollama Cloud",
};

export function getProvider(provider: ProviderKind): ProviderOption {
  const id = idProvider(provider);
  if (id === "ollama") return OLLAMA_PROVIDER;
  return PROVIDERS.find((item) => item.id === id) ?? PROVIDERS[0]!;
}

/** Fournisseurs distants — sans l'IA locale Candilog gérée par Ollama. */
export const OTHER_PROVIDERS: readonly ProviderOption[] = PROVIDERS.filter(
  (item) => item.id !== "candilog_local",
);

export function isCustomProvider(provider: ProviderKind): provider is { custom: string } {
  return typeof provider === "object" && provider !== null && "custom" in provider;
}

export function idProvider(provider: ProviderKind): ProviderOption["id"] {
  return isCustomProvider(provider) ? "custom" : provider;
}

export function toProvider(id: ProviderOption["id"]): ProviderKind {
  return id === "custom" ? { custom: "custom" } : id;
}

export function defaultEndpoint(id: ProviderOption["id"]): string | null {
  switch (id) {
    case "candilog_local":
      return null;
    case "ollama":
      return "http://localhost:11434";
    case "claude":
      return "https://api.anthropic.com";
    case "gemini":
      return "https://generativelanguage.googleapis.com";
    case "mistral":
      return "https://api.mistral.ai";
    case "deepseek":
      return "https://api.deepseek.com";
    case "openai":
    case "custom":
      return "https://api.openai.com";
  }
}

/** Aucun modèle n'est prérempli : l'utilisateur choisit après avoir sélectionné le fournisseur. */
export function defaultModel(_id: ProviderOption["id"]): string {
  void _id;
  return "";
}

/** Snapshot d'un formulaire LLM pour le mémoriser sous l'identifiant du fournisseur. */
export function presetFromLlm(llm: LlmForm): LlmProviderPresetForm {
  return {
    endpoint: llm.endpoint,
    model: llm.model,
    temperature: llm.temperature,
    mode: llm.mode,
    api_key_configured: llm.api_key_configured,
  };
}

/** Reconstruit le formulaire d'un fournisseur à partir de son preset, ou des défauts. */
export function llmFromPreset(
  id: ProviderOption["id"],
  preset: LlmProviderPresetForm | undefined,
  defaults?: { temperature?: number; mode?: AnalysisMode },
): LlmForm {
  return {
    provider: toProvider(id),
    endpoint: preset?.endpoint ?? defaultEndpoint(id),
    model: preset?.model ?? defaultModel(id),
    temperature: preset?.temperature ?? defaults?.temperature ?? 0.7,
    mode: preset?.mode ?? defaults?.mode ?? "auto",
    api_key_configured: preset?.api_key_configured ?? false,
  };
}
