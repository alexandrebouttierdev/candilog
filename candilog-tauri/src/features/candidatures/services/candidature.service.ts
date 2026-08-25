import { ipc } from "@/shared/services/ipc";
import type {
  Candidature,
  FiltreCandidatures,
  NouvelleCandidature,
  RepartitionPipeline,
  StatutCandidature,
} from "@/shared/types/generated/candidatures";
import type { Page } from "@/shared/types/page";

export type {
  Candidature,
  FiltreCandidatures,
  NouvelleCandidature,
  RepartitionPipeline,
  StatutCandidature,
};

/** Seule couche du frontend qui connaisse les commandes Tauri des candidatures. */
export const candidatureService = {
  listerPage: (params: { page: number; pageSize: number; filtre: FiltreCandidatures }) =>
    ipc<Page<Candidature>>("candidatures_lister_page", params),

  /** Compteurs des quatre colonnes du Kanban, calculés par SQLite. */
  repartition: (filtre: FiltreCandidatures) =>
    ipc<RepartitionPipeline>("candidatures_repartition", { filtre }),

  obtenir: (id: string) => ipc<Candidature>("candidatures_obtenir", { id }),

  creer: (input: NouvelleCandidature) => ipc<Candidature>("candidatures_creer", { input }),

  modifier: (id: string, input: NouvelleCandidature) =>
    ipc<Candidature>("candidatures_modifier", { id, input }),

  /** Change le seul statut — geste du glisser-déposer. */
  changerStatut: (id: string, statut: StatutCandidature) =>
    ipc<Candidature>("candidatures_changer_statut", { id, statut }),

  supprimer: (id: string) => ipc<void>("candidatures_supprimer", { id }),

  /** Écrit au chemin choisi tout le filtre courant, et renvoie le nombre de lignes. */
  exporterCsv: (filtre: FiltreCandidatures, chemin: string) =>
    ipc<number>("candidatures_exporter_csv", { filtre, chemin }),
};
