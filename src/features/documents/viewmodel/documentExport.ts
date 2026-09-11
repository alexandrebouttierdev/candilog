import type { CoverLetterExport, ResumeDocument } from "../services/documentsService";
import { documentsService } from "../services/documentsService";
import type { ToastMessage } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";

type Notify = (toast: Omit<ToastMessage, "id">) => void;

/** Export PDF d'un CV : toast de succès uniquement si le dialogue natif a confirmé. */
export async function exportResumePdf(document: ResumeDocument, notify: Notify): Promise<void> {
  try {
    const exported = await documentsService.exportPdf(document);
    if (!exported) return;
    notify({ tone: "success", title: "CV exporté" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}

/** Export PDF d'une lettre de motivation. */
export async function exportCoverLetterPdf(
  cover_letter: CoverLetterExport,
  notify: Notify,
): Promise<void> {
  try {
    const exported = await documentsService.exportCoverLetterPdf(cover_letter);
    if (!exported) return;
    notify({ tone: "success", title: "Lettre exportée" });
  } catch (error) {
    notify({
      tone: "error",
      title: "Export PDF impossible",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}
