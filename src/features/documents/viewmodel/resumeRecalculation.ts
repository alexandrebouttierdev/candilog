import type { ResumeWorkspace } from "@/shared/types/generated/documents";

interface ResumeRecalculationOptions {
  workspace: ResumeWorkspace;
  recalculate: (workspace: ResumeWorkspace) => Promise<ResumeWorkspace>;
  isCurrent: () => boolean;
  onSuccess: (workspace: ResumeWorkspace) => void;
  onError: (error: unknown) => void;
  onSettled: () => void;
}

/** Exécute un recalcul sans publier une réponse devenue obsolète ou démontée. */
export async function runResumeRecalculation({
  workspace,
  recalculate,
  isCurrent,
  onSuccess,
  onError,
  onSettled,
}: ResumeRecalculationOptions): Promise<void> {
  try {
    const updated = await recalculate(workspace);
    if (!isCurrent()) return;
    onSuccess(updated);
  } catch (error) {
    if (!isCurrent()) return;
    onError(error);
  } finally {
    if (isCurrent()) onSettled();
  }
}
