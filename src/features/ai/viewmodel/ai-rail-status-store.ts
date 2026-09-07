import { create } from "zustand";
import type { TestConnexion } from "@/features/settings/model/etatIa";

interface AiRailStatusState {
  connectionTest: TestConnexion;
  lastOperationFailed: boolean;
  setConnectionTest: (test: TestConnexion) => void;
  setLastOperationFailed: (failed: boolean) => void;
}

/** État d’erreur / test connu partagé avec le rail (pas de ping automatique). */
export const useAiRailStatusStore = create<AiRailStatusState>((set) => ({
  connectionTest: "idle",
  lastOperationFailed: false,
  setConnectionTest: (connectionTest) => set({ connectionTest }),
  setLastOperationFailed: (lastOperationFailed) => set({ lastOperationFailed }),
}));
