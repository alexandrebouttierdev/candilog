import { ipc } from "@/shared/services/ipc";
import type {
  Application,
  ApplicationFilter,
  NewApplication,
  PipelineBreakdown,
  ApplicationStatus,
} from "@/shared/types/generated/applications";
import type { Page } from "@/shared/types/page";

export type {
  Application,
  ApplicationFilter,
  NewApplication,
  PipelineBreakdown,
  ApplicationStatus,
};

/** Seule couche du frontend qui connaisse les commandes Tauri des candidatures. */
export const applicationService = {
  listPage: (params: { page: number; page_size: number; filter: ApplicationFilter }) =>
    ipc<Page<Application>>("applications_list_page", params),

  /** Compteurs des quatre colonnes du Kanban, calculés par SQLite. */
  breakdown: (filter: ApplicationFilter) =>
    ipc<PipelineBreakdown>("applications_breakdown", { filter }),

  get: (id: string) => ipc<Application>("applications_get", { id }),

  create: (input: NewApplication) => ipc<Application>("applications_create", { input }),

  update: (id: string, input: NewApplication) =>
    ipc<Application>("applications_update", { id, input }),

  /** Change le seul statut — geste du glisser-déposer. */
  changeStatus: (id: string, status: ApplicationStatus) =>
    ipc<Application>("applications_change_status", { id, status }),

  delete: (id: string) => ipc<void>("applications_delete", { id }),

  /** Écrit au chemin choisi tout le filtre courant, et renvoie le nombre de lignes. */
  exportCsv: (filter: ApplicationFilter, path: string) =>
    ipc<number>("applications_export_csv", { filter, path }),
};
