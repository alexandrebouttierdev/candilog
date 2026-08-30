import { ipc } from "@/shared/services/ipc";
import type { Company, CompanyFilter, NewCompany } from "@/shared/types/generated/companies";
import type { Page } from "@/shared/types/page";

export type { Company, CompanyFilter, NewCompany };

/**
 * Seule couche du frontend qui connaisse les commandes Tauri des entreprises.
 *
 * Les ViewModels l'appellent, les vues jamais (MIGRATION.md §7). Chercher les appelants de
 * ce module donne la liste exhaustive de ce que l'interface fait du répertoire.
 */
export const companyService = {
  /** Toutes les entreprises, pour alimenter un sélecteur. */
  list: () => ipc<Company[]>("companies_list"),

  /** Une page du répertoire, filtrée par recherche libre, secteur, type et taille. */
  listPage: (params: { page: number; page_size: number; filter: CompanyFilter }) =>
    ipc<Page<Company>>("companies_list_page", params),

  get: (id: string) => ipc<Company>("companies_get", { id }),

  create: (input: NewCompany) => ipc<Company>("companies_create", { input }),

  update: (id: string, input: NewCompany) =>
    ipc<Company>("companies_update", { id, input }),

  delete: (id: string) => ipc<void>("companies_delete", { id }),
};
