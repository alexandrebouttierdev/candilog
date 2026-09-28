import { ipc } from "@/shared/services/ipc";
import type { NewSavedView, SavedView } from "@/shared/types/generated/views";

export type { NewSavedView, SavedView };

/** Seule couche du frontend qui connaisse les commandes Tauri des vues enregistrées. */
export const viewsService = {
  list: () => ipc<SavedView[]>("saved_views_list"),
  create: (input: NewSavedView) => ipc<SavedView>("saved_views_create", { input }),
  update: (id: string, input: NewSavedView) => ipc<SavedView>("saved_views_update", { id, input }),
  duplicate: (id: string) => ipc<SavedView>("saved_views_duplicate", { id }),
  delete: (id: string) => ipc<void>("saved_views_delete", { id }),
};
