import { useState } from "react";
import {
  aiService,
  isAiNotConfiguredError,
  useAiOperation,
  useAiProgress,
  useAiRailStatusStore,
  useAiTimer,
  type AiExecution,
  type ImportedResumeAnalysis,
  type SelectedResumeFile,
} from "@/features/ai";
import { documentsService } from "../services/documentsService";
import { AppError } from "@/shared/types/app-error";

function message(error: unknown): string {
  return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite.";
}

/** Orchestration de l'analyse d'un CV PDF importé contre une offre. */
export function useResumeAnalysisViewModel() {
  const [jobOffer, setJobOffer] = useState("");
  const [selectedFile, setSelectedFile] = useState<SelectedResumeFile | null>(null);
  const [selecting, setSelecting] = useState(false);
  const { operation, stopping, start, stop, finish, isCurrent } = useAiOperation();
  const [result, setResult] = useState<ImportedResumeAnalysis | null>(null);
  const [metrics, setMetrics] = useState<Pick<
    AiExecution<unknown>,
    "elapsed_ms" | "tokens_used"
  > | null>(null);
  const [error, setError] = useState<string | null>(null);
  const progress = useAiProgress(stopping ? null : (operation?.id ?? null));
  const timer = useAiTimer(operation !== null && !stopping);

  async function selectFile(): Promise<void> {
    setSelecting(true);
    setError(null);
    try {
      const selected = await aiService.selectResumeFile();
      if (selected !== null) {
        setSelectedFile(selected);
        setResult(null);
        setMetrics(null);
      }
    } catch (caught) {
      setError(message(caught));
    } finally {
      setSelecting(false);
    }
  }

  function reset(): void {
    if (operation !== null) return;
    setSelectedFile(null);
    setResult(null);
    setMetrics(null);
    setError(null);
    setJobOffer("");
  }

  async function run(): Promise<void> {
    if (!selectedFile) {
      setError("Choisissez le CV PDF à analyser.");
      return;
    }
    if (!jobOffer.trim()) {
      setError("Collez l’offre ciblée avant de lancer l’analyse.");
      return;
    }
    let id: string;
    try {
      id = start("analyse");
    } catch (caught) {
      if (isAiNotConfiguredError(caught)) return;
      setError(message(caught));
      return;
    }
    setError(null);
    timer.start();
    try {
      const execution = await aiService.analyzeResume({
        generation_id: id,
        job_offer: jobOffer,
      });
      if (!isCurrent(id)) return;
      timer.stop();
      setResult(execution.output);
      setMetrics({
        elapsed_ms: execution.elapsed_ms,
        tokens_used: execution.tokens_used,
      });
    } catch (caught) {
      if (isCurrent(id) && !(caught instanceof AppError && caught.code === "CANCELLED")) {
        setError(message(caught));
        useAiRailStatusStore.getState().setLastOperationFailed(true);
      }
    } finally {
      finish(id);
    }
  }

  async function stopAnalysis(): Promise<void> {
    try {
      await stop();
    } catch (caught) {
      setError(message(caught));
    }
  }

  const canReset =
    operation === null &&
    (selectedFile !== null ||
      result !== null ||
      jobOffer.trim().length > 0 ||
      error !== null);

  return {
    jobOffer,
    setJobOffer,
    selectedFile,
    selecting,
    operation,
    stopping,
    result,
    metrics,
    error,
    progress,
    elapsedMs: timer.elapsedMs,
    canReset,
    selectFile,
    reset,
    run,
    stop: stopAnalysis,
    readClipboard: () => documentsService.readClipboard(),
  };
}
