import { ipc } from "@/shared/services/ipc";
import type { Entretien, NouvelEntretien } from "@/shared/types/generated/entretiens";

export type { Entretien, NouvelEntretien };

/** Seule couche du frontend qui connaisse les commandes Tauri des entretiens. */
export const entretienService = {
  /** Entretiens d'une plage de dates, bornes incluses. */
  listerEntre: (from: string, to: string) =>
    ipc<Entretien[]>("entretiens_lister_entre", { from, to }),

  obtenir: (id: string) => ipc<Entretien>("entretiens_obtenir", { id }),

  /**
   * Enregistre et fait avancer la candidature au statut « Entretien ».
   *
   * `id` absent crée, `id` présent modifie : le chemin est unique côté Rust, où l'écriture
   * et la mise à jour du statut tiennent dans la même transaction.
   */
  enregistrer: (id: string | null, input: NouvelEntretien) =>
    ipc<Entretien>("entretiens_enregistrer", { id, input }),

  supprimer: (id: string) => ipc<void>("entretiens_supprimer", { id }),
};
