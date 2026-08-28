import { ipc } from "@/shared/services/ipc";
import type { AnalyseCvImporte, AnalyseOffre, DemandeAnalyseCv, DemandeGenerationCv, DemandeImportProfil, DemandeLettre, GenerationCv, ProfilExtrait } from "../model/types";

export const iaService = {
  analyserOffre: (texte: string) => ipc<AnalyseOffre>("ia_analyser_offre", { texte }),
  genererCv: (demande: DemandeGenerationCv) => ipc<GenerationCv>("ia_generer_cv", { demande }),
  genererLettre: (demande: DemandeLettre) => ipc<string>("ia_generer_lettre", { demande }),
  analyserCv: (demande: DemandeAnalyseCv) => ipc<AnalyseCvImporte>("ia_analyser_cv", { demande }),
  importerProfil: (demande: DemandeImportProfil) => ipc<ProfilExtrait>("ia_importer_profil", { demande }),
  annuler: (generationId: string) => ipc<void>("ia_annuler", { generationId }),
};

export function generationId(): string { return crypto.randomUUID(); }
