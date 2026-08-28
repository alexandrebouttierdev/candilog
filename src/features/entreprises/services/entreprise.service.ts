import { ipc } from "@/shared/services/ipc";
import type { Entreprise, NouvelleEntreprise } from "@/shared/types/generated/entreprises";
import type { Page } from "@/shared/types/page";

export type { Entreprise, NouvelleEntreprise };

/**
 * Seule couche du frontend qui connaisse les commandes Tauri des entreprises.
 *
 * Les ViewModels l'appellent, les vues jamais (MIGRATION.md §7). Chercher les appelants de
 * ce module donne la liste exhaustive de ce que l'interface fait du répertoire.
 */
export const entrepriseService = {
  /** Toutes les entreprises, pour alimenter un sélecteur. */
  lister: () => ipc<Entreprise[]>("entreprises_lister"),

  /** Une page du répertoire, filtrée par recherche libre et par type. */
  listerPage: (params: {
    page: number;
    pageSize: number;
    search: string;
    companyType: string | null;
  }) =>
    ipc<Page<Entreprise>>("entreprises_lister_page", {
      page: params.page,
      pageSize: params.pageSize,
      search: params.search,
      companyType: params.companyType,
    }),

  /** Types réellement présents, pour alimenter le filtre. */
  listerTypes: () => ipc<string[]>("entreprises_lister_types"),

  obtenir: (id: string) => ipc<Entreprise>("entreprises_obtenir", { id }),

  creer: (input: NouvelleEntreprise) => ipc<Entreprise>("entreprises_creer", { input }),

  modifier: (id: string, input: NouvelleEntreprise) =>
    ipc<Entreprise>("entreprises_modifier", { id, input }),

  supprimer: (id: string) => ipc<void>("entreprises_supprimer", { id }),
};
