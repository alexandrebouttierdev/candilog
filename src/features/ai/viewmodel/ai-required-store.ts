import { create } from "zustand";
import type { LlmForm } from "@/shared/types/generated/settings";
import { iaEstConfiguree } from "@/features/settings/model/etatIa";

interface AiRequiredState {
  /** Dernière configuration connue ; `null` tant que les réglages n'ont pas été chargés. */
  llm: LlmForm | null;
  open: boolean;
  setLlm: (llm: LlmForm | null) => void;
  show: () => void;
  hide: () => void;
  /** `true` si un fournisseur et un modèle sont prêts ; `null` si encore inconnu. */
  configured: () => boolean | null;
}

/**
 * Pont entre les réglages et le garde-fou des opérations IA.
 *
 * Le client Query n'est pas accessible hors React : ce store reçoit la configuration
 * dès qu'elle est chargée, et `useAiOperation` s'y fie de façon synchrone.
 */
export const useAiRequiredStore = create<AiRequiredState>((set, get) => ({
  llm: null,
  open: false,
  setLlm: (llm) => set({ llm }),
  show: () => set({ open: true }),
  hide: () => set({ open: false }),
  configured: () => {
    const { llm } = get();
    if (llm === null) return null;
    return iaEstConfiguree(llm);
  },
}));
