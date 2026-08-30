import { ipc } from "@/shared/services/ipc";
import type { ResumeSummary, ResumeVersion, CoverLetterExport, CoverLetter, NewResume, NewCoverLetter } from "@/shared/types/generated/documents";
import type { GeneratedResume } from "@/shared/types/generated/ai";
import type { Page } from "@/shared/types/page";

export type * from "@/shared/types/generated/documents";

export const documentsService = {
  /** Texte du presse-papiers : la webview ne sait pas le lire, le natif si. */
  readClipboard: () => ipc<string>("documents_read_clipboard"),
  listResume: () => ipc<ResumeSummary[]>("documents_resume_list"),
  listResumePage: (params: { page: number; page_size: number; search: string }) =>
    ipc<Page<ResumeSummary>>("documents_resume_list_page", params),
  getResume: (id: string) => ipc<ResumeVersion>("documents_resume_get", { id }),
  saveResume: (input: NewResume) => ipc<ResumeVersion>("documents_resume_save", { input }),
  deleteResume: (id: string) => ipc<void>("documents_resume_delete", { id }),
  exportPdf: (resume: GeneratedResume) =>
    ipc<boolean>("documents_resume_export_pdf", { resume }),
  exportCoverLetterPdf: (cover_letter: CoverLetterExport) =>
    ipc<boolean>("documents_cover_letter_export_pdf", { cover_letter }),
  listCoverLetters: () => ipc<CoverLetter[]>("documents_cover_letters_list"),
  listCoverLettersPage: (params: { page: number; page_size: number; search: string }) =>
    ipc<Page<CoverLetter>>("documents_cover_letters_list_page", params),
  getCoverLetter: (id: string) => ipc<CoverLetter>("documents_cover_letter_get", { id }),
  saveCoverLetter: (input: NewCoverLetter) => ipc<CoverLetter>("documents_cover_letter_save", { input }),
  deleteCoverLetter: (id: string) => ipc<void>("documents_cover_letter_delete", { id }),
};
