import { ipc } from "@/shared/services/ipc";
import type { SecteurActivite } from "@/shared/types/generated/secteurs";

export type { SecteurActivite };

/**
 * Référentiel des secteurs d'activité.
 *
 * Lecture seule : le référentiel est garanti par le backend au démarrage, l'utilisateur ne
 * l'alimente pas.
 */
export const secteurService = {
  lister: () => ipc<SecteurActivite[]>("secteurs_lister"),
};
