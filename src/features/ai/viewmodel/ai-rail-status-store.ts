import { create } from "zustand";
import type { ConnectionTest } from "@/features/settings";

interface AiRailStatusState {
  connectionTest: ConnectionTest;
  lastOperationFailed: boolean;
  setConnectionTest: (test: ConnectionTest) => void;
  setLastOperationFailed: (failed: boolean) => void;
}

/** État d’erreur / test connu partagé avec le rail (pas de ping automatique). */
export const useAiRailStatusStore = create<AiRailStatusState>((set) => ({
  connectionTest: "idle",
  lastOperationFailed: false,
  setConnectionTest: (connectionTest) => set({ connectionTest }),
  setLastOperationFailed: (lastOperationFailed) => set({ lastOperationFailed }),
}));
