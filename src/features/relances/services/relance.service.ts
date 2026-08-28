import { ipc } from "@/shared/services/ipc";
import type { NouvelleRelance, Relance } from "@/shared/types/generated/relances";

export type { NouvelleRelance, Relance };

/** Seule couche du frontend qui connaisse les commandes Tauri des relances. */
export const relanceService = {
  /** Relances d'une plage de dates, bornes incluses. */
  listerEntre: (from: string, to: string) =>
    ipc<Relance[]>("relances_lister_entre", { from, to }),

  creer: (input: NouvelleRelance) => ipc<Relance>("relances_creer", { input }),

  modifier: (id: string, input: NouvelleRelance) =>
    ipc<Relance>("relances_modifier", { id, input }),

  supprimer: (id: string) => ipc<void>("relances_supprimer", { id }),
};
