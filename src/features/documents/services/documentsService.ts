import { ipc } from "@/shared/services/ipc";
import type { ResumeSummary, ResumeVersion, CoverLetterExport, CoverLetter, NewResume, NewCoverLetter } from "@/shared/types/generated/documents";
import type { GeneratedResume } from "@/shared/types/generated/ai";

export type * from "@/shared/types/generated/documents";

export const documentsService = {
  listResume: () => ipc<ResumeSummary[]>("documents_resume_list"),
  getResume: (id: string) => ipc<ResumeVersion>("documents_resume_get", { id }),
  saveResume: (input: NewResume) => ipc<ResumeVersion>("documents_resume_save", { input }),
  deleteResume: (id: string) => ipc<void>("documents_resume_delete", { id }),
  exportPdf: (resume: GeneratedResume, path: string) =>
    ipc<void>("documents_resume_export_pdf", { resume, path }),
  exportCoverLetterPdf: (cover_letter: CoverLetterExport, path: string) =>
    ipc<void>("documents_cover_letter_export_pdf", { cover_letter, path }),
  listCoverLetters: () => ipc<CoverLetter[]>("documents_cover_letters_list"),
  getCoverLetter: (id: string) => ipc<CoverLetter>("documents_cover_letter_get", { id }),
  saveCoverLetter: (input: NewCoverLetter) => ipc<CoverLetter>("documents_cover_letter_save", { input }),
  deleteCoverLetter: (id: string) => ipc<void>("documents_cover_letter_delete", { id }),
};
