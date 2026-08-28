import { ipc } from "@/shared/services/ipc";
import type { ActivitySector } from "@/shared/types/generated/sectors";

export type { ActivitySector };

/**
 * Référentiel des secteurs d'activité.
 *
 * Lecture seule : le référentiel est garanti par le backend au démarrage, l'utilisateur ne
 * l'alimente pas.
 */
export const sectorService = {
  list: () => ipc<ActivitySector[]>("sectors_list"),
};
