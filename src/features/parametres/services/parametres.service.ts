import { ipc } from "@/shared/services/ipc";
import type {
  APropos,
  LlmFormulaire,
  MiseAJour,
  Parametres,
} from "@/shared/types/generated/parametres";

export type * from "@/shared/types/generated/parametres";

/** Frontière IPC des réglages, sauvegardes et mises à jour. */
export const parametresService = {
  charger: () => ipc<Parametres>("parametres_charger"),
  enregistrer: (parametres: Parametres) =>
    ipc<Parametres>("parametres_enregistrer", { parametres }),
  testerConnexion: (llm: LlmFormulaire) =>
    ipc<void>("parametres_tester_connexion", { llm }),
  listerModeles: (llm: LlmFormulaire) =>
    ipc<string[]>("parametres_lister_modeles", { llm }),
  viderCacheIa: () => ipc<void>("parametres_vider_cache_ia"),
  exporter: (chemin: string) => ipc<void>("parametres_exporter", { chemin }),
  restaurer: (chemin: string) => ipc<void>("parametres_restaurer", { chemin }),
  reinitialiser: () => ipc<void>("parametres_reinitialiser"),
  verifierMaj: () => ipc<MiseAJour | null>("parametres_verifier_maj"),
  telechargerMaj: (url: string, nom: string) =>
    ipc<string>("parametres_telecharger_maj", { url, nom }),
  aPropos: () => ipc<APropos>("parametres_a_propos"),
};
