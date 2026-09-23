import { ipc } from "@/shared/services/ipc";
import type {
  ResumeSummary,
  ResumeVersion,
  CoverLetterExport,
  CoverLetter,
  NewResume,
  NewCoverLetter,
  ResumeWorkspace,
  ResumeDocument,
} from "@/shared/types/generated/documents";
import type { ResumeGeneration } from "@/shared/types/generated/ai";
import type { Page } from "@/shared/types/page";

export type * from "@/shared/types/generated/documents";

export const documentsService = {
  /** Texte du presse-papiers : la webview ne sait pas le lire, le natif si. */
  readClipboard: () => ipc<string>("documents_read_clipboard"),
  /** `scored_only` : seulement les CV qui portent un score ATS (onglet « Analyses »). */
  listResumePage: (params: { page: number; page_size: number; search: string; scored_only?: boolean }) =>
    ipc<Page<ResumeSummary>>("documents_resume_list_page", { scored_only: false, ...params }),
  getResume: (id: string) => ipc<ResumeVersion>("documents_resume_get", { id }),
  saveResume: (input: NewResume) => ipc<ResumeVersion>("documents_resume_save", { input }),
  deleteResume: (id: string) => ipc<void>("documents_resume_delete", { id }),
  prepareResume: (generation: ResumeGeneration) =>
    ipc<ResumeWorkspace>("documents_resume_prepare", { generation }),
  recalculateResume: (workspace: ResumeWorkspace) =>
    ipc<ResumeWorkspace>("documents_resume_recalculate", { workspace }),
  applyResumeProposal: (workspace: ResumeWorkspace, proposal_id: string) =>
    ipc<ResumeWorkspace>("documents_resume_apply_proposal", { workspace, proposal_id }),
  rejectResumeProposal: (workspace: ResumeWorkspace, proposal_id: string) =>
    ipc<ResumeWorkspace>("documents_resume_reject_proposal", { workspace, proposal_id }),
  exportPdf: (document: ResumeDocument) =>
    ipc<boolean>("documents_resume_export_pdf", { document }),
  exportCoverLetterPdf: (cover_letter: CoverLetterExport) =>
    ipc<boolean>("documents_cover_letter_export_pdf", { cover_letter }),
  listCoverLettersPage: (params: { page: number; page_size: number; search: string }) =>
    ipc<Page<CoverLetter>>("documents_cover_letters_list_page", params),
  getCoverLetter: (id: string) => ipc<CoverLetter>("documents_cover_letter_get", { id }),
  saveCoverLetter: (input: NewCoverLetter) => ipc<CoverLetter>("documents_cover_letter_save", { input }),
  deleteCoverLetter: (id: string) => ipc<void>("documents_cover_letter_delete", { id }),
};
