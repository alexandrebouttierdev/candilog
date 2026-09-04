import { playCompletionSound } from "@/shared/lib/completion-sound";
import { ipc } from "@/shared/services/ipc";
import type { AiExecution, ImportedResumeAnalysis, ListingAnalysis, ResumeAnalysisRequest, ResumeGenerationRequest, ProfileImportRequest, CoverLetterRequest, ResumeGeneration, SelectedResumeFile } from "../model/types";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";

/**
 * Signale la fin d'un traitement IA.
 *
 * Le point d'annonce est ici plutôt que dans chaque écran : toutes les générations passent
 * par ce service, et aucun appel ne peut donc être oublié. Un résultat `null` — sélecteur de
 * fichier annulé — comme une erreur restent muets : rien n'a été produit.
 */
function announce<T>(run: Promise<T>): Promise<T> {
  return run.then((value) => {
    if (value !== null) playCompletionSound();
    return value;
  });
}

export const aiService = {
  analyzeListing: (text: string) => announce(ipc<AiExecution<ListingAnalysis>>("ai_analyze_listing", { text })),
  generateResume: (request: ResumeGenerationRequest) => announce(ipc<AiExecution<ResumeGeneration>>("ai_generate_resume", { request })),
  generateCoverLetter: (request: CoverLetterRequest) => announce(ipc<AiExecution<string>>("ai_generate_cover_letter", { request })),
  selectResumeFile: () => ipc<SelectedResumeFile | null>("ai_select_resume_file"),
  analyzeResume: (request: ResumeAnalysisRequest) => announce(ipc<AiExecution<ImportedResumeAnalysis>>("ai_analyze_resume", { request })),
  importProfile: (request: ProfileImportRequest) => announce(ipc<AiExecution<ImportProfilePreview> | null>("ai_import_profile", { request })),
  cancel: (generation_id: string) => ipc<void>("ai_cancel", { generation_id }),
};

export function generation_id(): string { return crypto.randomUUID(); }
