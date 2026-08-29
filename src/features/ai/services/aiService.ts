import { ipc } from "@/shared/services/ipc";
import type { ImportedResumeAnalysis, ListingAnalysis, ResumeAnalysisRequest, ResumeGenerationRequest, ProfileImportRequest, CoverLetterRequest, ResumeGeneration } from "../model/types";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";

export const aiService = {
  analyzeListing: (text: string) => ipc<ListingAnalysis>("ai_analyze_listing", { text }),
  generateResume: (request: ResumeGenerationRequest) => ipc<ResumeGeneration>("ai_generate_resume", { request }),
  generateCoverLetter: (request: CoverLetterRequest) => ipc<string>("ai_generate_cover_letter", { request }),
  analyzeResume: (request: ResumeAnalysisRequest) => ipc<ImportedResumeAnalysis>("ai_analyze_resume", { request }),
  importProfile: (request: ProfileImportRequest) => ipc<ImportProfilePreview>("ai_import_profile", { request }),
  cancel: (generation_id: string) => ipc<void>("ai_cancel", { generation_id }),
};

export function generation_id(): string { return crypto.randomUUID(); }
