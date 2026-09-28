import { ipc } from "@/shared/services/ipc";
import type { RelationsExport } from "@/shared/types/generated/companies";
import type {
  HistoryEntry,
  NewRelationNote,
  RelationNote,
  RelationRef,
} from "@/shared/types/generated/relations";

export const relationsService = {
  /** Deux CSV, entreprises puis contacts, dans le dossier choisi ; `null` si l'on annule. */
  exportCsv: () => ipc<RelationsExport | null>("relations_export_csv"),
  /** Historique d'une fiche, du plus récent au plus ancien. */
  history: (relation: RelationRef) => ipc<HistoryEntry[]>("relations_history", { relation }),
  addNote: (input: NewRelationNote) => ipc<RelationNote>("relations_add_note", { input }),
  deleteNote: (id: string) => ipc<void>("relations_delete_note", { id }),
};
