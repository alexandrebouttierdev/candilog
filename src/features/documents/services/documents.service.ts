import { ipc } from "@/shared/services/ipc";
import type { CvResume, CvVersion, Lettre, NouveauCv, NouvelleLettre } from "@/shared/types/generated/documents";
import type { CvGenere } from "@/shared/types/generated/ia";

export type * from "@/shared/types/generated/documents";

export const documentsService = {
  listerCv: () => ipc<CvResume[]>("documents_cv_lister"),
  obtenirCv: (id: string) => ipc<CvVersion>("documents_cv_obtenir", { id }),
  enregistrerCv: (input: NouveauCv) => ipc<CvVersion>("documents_cv_enregistrer", { input }),
  supprimerCv: (id: string) => ipc<void>("documents_cv_supprimer", { id }),
  exporterPdf: (cv: CvGenere, chemin: string) =>
    ipc<void>("documents_cv_exporter_pdf", { cv, chemin }),
  listerLettres: () => ipc<Lettre[]>("documents_lettres_lister"),
  obtenirLettre: (id: string) => ipc<Lettre>("documents_lettre_obtenir", { id }),
  enregistrerLettre: (input: NouvelleLettre) => ipc<Lettre>("documents_lettre_enregistrer", { input }),
  supprimerLettre: (id: string) => ipc<void>("documents_lettre_supprimer", { id }),
};
