import { ipc } from "@/shared/services/ipc";
import type { NewFollowUp, FollowUp } from "@/shared/types/generated/followUps";

export type { NewFollowUp, FollowUp };

/** Seule couche du frontend qui connaisse les commandes Tauri des relances. */
export const followUpService = {
  /** FollowUps d'une plage de dates, bornes incluses. */
  listBetween: (from: string, to: string) =>
    ipc<FollowUp[]>("follow_ups_list_between", { from, to }),

  create: (input: NewFollowUp) => ipc<FollowUp>("follow_ups_create", { input }),

  update: (id: string, input: NewFollowUp) =>
    ipc<FollowUp>("follow_ups_update", { id, input }),

  delete: (id: string) => ipc<void>("follow_ups_delete", { id }),
};
