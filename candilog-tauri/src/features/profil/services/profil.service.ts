import { ipc } from "@/shared/services/ipc";
import type { Profil, ProfilCharge } from "@/shared/types/generated/profil";

export type * from "@/shared/types/generated/profil";

/** Frontière IPC du profil professionnel. */
export const profilService = {
  charger: () => ipc<ProfilCharge>("profil_charger"),
  enregistrer: (profil: Profil) => ipc<ProfilCharge>("profil_enregistrer", { profil }),
};
