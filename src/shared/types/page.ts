import type { Page } from "./generated/page";

export type { Page };

/** Size de page par défaut des listes maîtresses, alignée sur les maquettes. */
export const PAGE_SIZE = 8;
/** Liste des entreprises : dix fiches par page, la hauteur de la grille les accueille. */
export const COMPANIES_PAGE_SIZE = 10;
/** Page unique du Kanban : assez large pour les quatre colonnes, plafonnée côté Rust. */
export const KANBAN_PAGE_SIZE = 500;
