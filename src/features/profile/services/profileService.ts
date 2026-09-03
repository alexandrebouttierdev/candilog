import { ipc } from "@/shared/services/ipc";
import type {
  ImportProfileRequest,
  ImportProfileResult,
  Profile,
  ProfilePayload,
} from "@/shared/types/generated/profile";

export type * from "@/shared/types/generated/profile";

/** Frontière IPC du profil professionnel. */
export const profileService = {
  load: () => ipc<ProfilePayload>("profile_load"),
  save: (profile: Profile) => ipc<ProfilePayload>("profile_save", { profile }),
  applyImport: (request: ImportProfileRequest) =>
    ipc<ImportProfileResult>("profile_apply_import", { request }),
  addSkill: (name: string) => ipc<ProfilePayload>("profile_add_skill", { name }),

  /** Ouvre le sélecteur natif ; `null` si l'utilisateur annule. */
  setPhoto: () => ipc<ProfilePayload | null>("profile_set_photo"),

  removePhoto: () => ipc<ProfilePayload>("profile_remove_photo"),

  /** Photo encodée en `data:` URL, ou `null` si le profil n'en a pas. */
  photo: () => ipc<string | null>("profile_photo"),

  /** Vide le seul profil : aucune autre donnée n'est touchée. */
  reset: () => ipc<ProfilePayload>("profile_reset"),
};
