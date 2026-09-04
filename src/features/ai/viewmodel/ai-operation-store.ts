import { create } from "zustand";

export type AiOperationKind = "analyse" | "generation" | "import";

export interface AiOperation {
  readonly id: string;
  readonly kind: AiOperationKind;
  readonly stopping: boolean;
  readonly stop: () => Promise<void>;
}

interface AiOperationState {
  active: AiOperation | null;
  begin: (operation: Omit<AiOperation, "stopping">) => void;
  markStopping: (id: string, stopping?: boolean) => void;
  finish: (id: string) => void;
}

/** État transverse d'une opération IA, limité à une opération active. */
export const useAiOperationStore = create<AiOperationState>((set) => ({
  active: null,

  begin: (operation) => set({ active: { ...operation, stopping: false } }),

  markStopping: (id, stopping = true) =>
    set((state) =>
      state.active?.id === id
        ? { active: { ...state.active, stopping } }
        : state,
    ),

  finish: (id) =>
    set((state) => (state.active?.id === id ? { active: null } : state)),
}));
