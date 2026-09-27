import { ipc } from "@/shared/services/ipc";
import type { RelationsExport } from "@/shared/types/generated/companies";

export const relationsService = {
  /** Deux CSV, entreprises puis contacts, dans le dossier choisi ; `null` si l'on annule. */
  exportCsv: () => ipc<RelationsExport | null>("relations_export_csv"),
};
