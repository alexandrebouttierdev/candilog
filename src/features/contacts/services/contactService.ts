import { ipc } from "@/shared/services/ipc";
import type { Contact, NewContact } from "@/shared/types/generated/contacts";
import type { Page } from "@/shared/types/page";

export type { Contact, NewContact };

/**
 * Seule couche du frontend qui connaisse les commandes Tauri des contacts.
 */
export const contactService = {
  list: () => ipc<Contact[]>("contacts_list"),

  listPage: (params: { page: number; page_size: number; search: string }) =>
    ipc<Page<Contact>>("contacts_list_page", params),

  get: (id: string) => ipc<Contact>("contacts_get", { id }),

  create: (input: NewContact) => ipc<Contact>("contacts_create", { input }),

  update: (id: string, input: NewContact) =>
    ipc<Contact>("contacts_update", { id, input }),

  delete: (id: string) => ipc<void>("contacts_delete", { id }),
};
