import type { ProviderKind } from "@/shared/types/generated/settings";

export interface FournisseurOption {
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
export const FOURNISSEURS: readonly FournisseurOption[] = [
  {
    id: "candilog_local",
    label: "IA locale Candilog",
    hint: "Modèles gérés localement",
    recommended: true,
  },
  { id: "ollama", label: "Ollama", hint: "Votre installation ou Ollama Cloud" },
  { id: "claude", label: "Claude", hint: "Anthropic" },
  { id: "openai", label: "OpenAI", hint: "GPT" },
  { id: "gemini", label: "Gemini", hint: "Google" },
  { id: "mistral", label: "Mistral", hint: "Europe" },
  { id: "deepseek", label: "DeepSeek", hint: "API" },
  { id: "custom", label: "Personnalisé", hint: "Compatible OpenAI" },
];

/** Fournisseurs distants — sans l'IA locale Candilog gérée par Ollama. */
export const FOURNISSEURS_AUTRES: readonly FournisseurOption[] = FOURNISSEURS.filter(
  (item) => item.id !== "candilog_local",
);

export function estPersonnalise(provider: ProviderKind): provider is { custom: string } {
  return typeof provider === "object" && provider !== null && "custom" in provider;
}

export function idProvider(provider: ProviderKind): FournisseurOption["id"] {
  return estPersonnalise(provider) ? "custom" : provider;
}

export function versProvider(id: FournisseurOption["id"]): ProviderKind {
  return id === "custom" ? { custom: "custom" } : id;
}

export function endpointDefaut(id: FournisseurOption["id"]): string | null {
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
export function modelDefaut(_id: FournisseurOption["id"]): string {
  void _id;
  return "";
}
