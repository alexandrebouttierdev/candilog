import { useEffect, useRef, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { aiService, generation_id } from "@/features/ai/services/aiService";
import type { AiExecution, ResumeGeneration } from "@/features/ai/model/types";
import { useAiProgress, useCancelAiOnUnmount } from "@/features/ai/viewmodel/useAiProgress";
import { useAiTimer } from "@/features/ai/viewmodel/useAiTimer";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { ResumeWorkspace } from "@/shared/types/generated/documents";
import { documentsService } from "../services/documentsService";

const RESUME_KEY = ["documents", "cv"] as const;

export interface ResumeGeneratorInitial {
  result: ResumeGeneration | null;
  workspace: ResumeWorkspace | null;
  name: string;
}

function errorMessage(error: unknown): string {
  return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite.";
}

function errorDetail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Orchestration de la génération, préparation et sauvegarde d'un CV ciblé. */
export function useResumeGeneratorViewModel(initial: ResumeGeneratorInitial) {
  const mounted = useRef(true);
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [jobOffer, setJobOffer] = useState("");
  const [operation, setOperation] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [historical] = useState(initial.result);
  const [name, setName] = useState(initial.name);
  const [workspace, setWorkspace] = useState<ResumeWorkspace | null>(initial.workspace);
  const [generationIndex, setGenerationIndex] = useState(0);
  const [briefOpen, setBriefOpen] = useState(false);
  const [metrics, setMetrics] = useState<
    Pick<AiExecution<unknown>, "elapsed_ms" | "tokens_used"> | null
  >(null);
  const progress = useAiProgress(operation);
  useCancelAiOnUnmount(operation);
  const timer = useAiTimer(operation !== null);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  useEffect(() => {
    if (workspace !== null || historical === null) return;
    let cancelled = false;
    void documentsService
      .prepareResume(historical)
      .then((prepared) => {
        if (cancelled || !mounted.current) return;
        setWorkspace(prepared);
        setGenerationIndex((index) => index + 1);
      })
      .catch((caught) => {
        if (!cancelled && mounted.current) setError(errorMessage(caught));
      });
    return () => {
      cancelled = true;
    };
  }, [historical, workspace]);

  async function generate(): Promise<void> {
    if (!jobOffer.trim()) {
      setError("Collez le texte de l’offre à cibler.");
      return;
    }
    const id = generation_id();
    setOperation(id);
    setError(null);
    timer.start();
    try {
      const execution = await aiService.generateResume({ generation_id: id, job_offer: jobOffer });
      if (!mounted.current) return;
      const prepared = await documentsService.prepareResume(execution.output);
      if (!mounted.current) return;
      timer.stop();
      setWorkspace(prepared);
      setGenerationIndex((index) => index + 1);
      setBriefOpen(false);
      setMetrics({
        elapsed_ms: execution.elapsed_ms,
        tokens_used: execution.tokens_used,
      });
      setName(`CV — ${prepared.job_offer.title || "Version ciblée"}`);
    } catch (caught) {
      if (
        mounted.current &&
        !(caught instanceof AppError && caught.code === "CANCELLED")
      ) {
        setError(errorMessage(caught));
      }
    } finally {
      if (mounted.current) setOperation(null);
    }
  }

  const save = useMutation({
    mutationFn: (content: ResumeWorkspace) => documentsService.saveResume({ name, content }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "CV ajouté à la bibliothèque" });
    },
    onError: (caught: unknown) => {
      notify({
        tone: "error",
        title: "Enregistrement impossible",
        detail: errorDetail(caught),
      });
    },
  });

  return {
    jobOffer,
    operation,
    error,
    name,
    workspace,
    generationIndex,
    briefOpen,
    progress,
    elapsedMs: timer.elapsedMs,
    durationMs: timer.durationMs,
    metrics,
    isSaving: save.isPending,
    setJobOffer,
    setName,
    openBrief: () => setBriefOpen(true),
    closeBrief: () => setBriefOpen(false),
    generate,
    cancel: () => (operation ? aiService.cancel(operation) : Promise.resolve()),
    saveResume: save.mutateAsync,
  };
}
