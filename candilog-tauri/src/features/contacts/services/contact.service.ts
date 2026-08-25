import { ipc } from "@/shared/services/ipc";
import type { Contact, NouveauContact } from "@/shared/types/generated/contacts";
import type { Page } from "@/shared/types/page";

export type { Contact, NouveauContact };

/**
 * Seule couche du frontend qui connaisse les commandes Tauri des contacts.
 */
export const contactService = {
  lister: () => ipc<Contact[]>("contacts_lister"),

  listerPage: (params: { page: number; pageSize: number; search: string }) =>
    ipc<Page<Contact>>("contacts_lister_page", params),

  obtenir: (id: string) => ipc<Contact>("contacts_obtenir", { id }),

  creer: (input: NouveauContact) => ipc<Contact>("contacts_creer", { input }),

  modifier: (id: string, input: NouveauContact) =>
    ipc<Contact>("contacts_modifier", { id, input }),

  supprimer: (id: string) => ipc<void>("contacts_supprimer", { id }),
};
