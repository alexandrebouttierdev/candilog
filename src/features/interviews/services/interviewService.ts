import { ipc } from "@/shared/services/ipc";
import type { Interview, NewInterview } from "@/shared/types/generated/interviews";

export type { Interview, NewInterview };

/** Seule couche du frontend qui connaisse les commandes Tauri des entretiens. */
export const interviewService = {
  /** Interviews d'une plage de dates, bornes incluses. */
  listBetween: (from: string, to: string) =>
    ipc<Interview[]>("interviews_list_between", { from, to }),

  get: (id: string) => ipc<Interview>("interviews_get", { id }),

  /**
   * Enregistre et fait avancer la candidature au statut « Interview ».
   *
   * `id` absent crée, `id` présent modifie : le chemin est unique côté Rust, où l'écriture
   * et la mise à jour du statut tiennent dans la même transaction.
   */
  save: (id: string | null, input: NewInterview) =>
    ipc<Interview>("interviews_save", { id, input }),

  delete: (id: string) => ipc<void>("interviews_delete", { id }),
};
