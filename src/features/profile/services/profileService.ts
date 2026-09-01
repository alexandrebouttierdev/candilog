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
};
