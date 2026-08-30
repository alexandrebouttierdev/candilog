import { ipc } from "@/shared/services/ipc";
import type {
  ActivitySector,
  ReferenceItem,
  Referentials,
} from "@/shared/types/generated/referentials";

export type { ActivitySector, ReferenceItem, Referentials };

/**
 * Référentiels métier : secteurs, domaines professionnels, types d'entreprise, contrats.
 *
 * Lecture seule : les listes sont semées par `init_schema.sql`, l'utilisateur ne les
 * alimente pas. Les quatre arrivent ensemble — les formulaires et les filtres en ont besoin
 * en même temps, et quatre appels séparés multiplieraient les états de chargement.
 */
export const referentialService = {
  load: () => ipc<Referentials>("referentials_load"),
};
