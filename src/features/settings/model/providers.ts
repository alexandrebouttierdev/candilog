import type { ProviderKind } from "@/shared/types/generated/settings";

export interface FournisseurOption {
  readonly id: "ollama" | "claude" | "openai" | "gemini" | "mistral" | "nvidia" | "custom";
  readonly label: string;
  readonly hint: string;
}

/** Grid des sept fournisseurs, jamais un menu déroulant. */
export const FOURNISSEURS: readonly FournisseurOption[] = [
  { id: "ollama", label: "Ollama", hint: "Local, sans clé" },
  { id: "claude", label: "Claude", hint: "Anthropic" },
  { id: "openai", label: "OpenAI", hint: "GPT" },
  { id: "gemini", label: "Gemini", hint: "Google" },
  { id: "mistral", label: "Mistral", hint: "Europe" },
  { id: "nvidia", label: "NVIDIA", hint: "NIM" },
  { id: "custom", label: "Personnalisé", hint: "Compatible OpenAI" },
];

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
    case "ollama":
      return "http://localhost:11434";
    case "claude":
      return "https://api.anthropic.com";
    case "gemini":
      return "https://generativelanguage.googleapis.com";
    case "mistral":
      return "https://api.mistral.ai";
    case "nvidia":
      return "https://integrate.api.nvidia.com";
    case "openai":
    case "custom":
      return "https://api.openai.com";
  }
}

export function modelDefaut(id: FournisseurOption["id"]): string {
  switch (id) {
    case "ollama":
      return "llama3.2:3b";
    case "claude":
      return "claude-sonnet-4-0";
    case "openai":
      return "gpt-4o";
    case "gemini":
      return "gemini-2.0-flash";
    case "mistral":
      return "mistral-small-latest";
    case "nvidia":
      return "meta/llama-3.1-70b-instruct";
    case "custom":
      return "";
  }
}
