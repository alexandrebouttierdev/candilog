import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  aiService,
  isAiNotConfiguredError,
  useAiOperation,
  useAiProgress,
  useAiRailStatusStore,
  useAiTimer,
} from "@/features/ai";
import { formatAiSummary } from "@/shared/lib/duration";
import { documentsService, type CoverLetter } from "../services/documentsService";
import { applyLetterCorrection, letterCorrectionFields } from "../model/letterMarkup";
import { PROFILE_KEY, profileService } from "@/features/profile";
import type { Identity } from "@/shared/types/generated/profile";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { exportCoverLetterPdf } from "./documentExport";
import { COVER_LETTERS_KEY } from "./documentKeys";

/** Nombre de consignes réinjectées : au-delà, le brief devient illisible pour le modèle. */
const MAX_CONSIGNES = 8;

export type LetterExchange = { auteur: "vous" | "candilog"; texte: string };

function message(error: unknown): string {
  return error instanceof AppError ? error.message : "Une erreur inattendue s’est produite.";
}

function detail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Orchestration de la rédaction, itération, correction et enregistrement d'une lettre. */
export function useLetterWriterViewModel(initial: CoverLetter | null) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [company, setCompany] = useState(initial?.company ?? "");
  const [jobTitle, setJobTitle] = useState(initial?.job_title ?? "");
  const [recipient, setRecipient] = useState(initial?.recipient ?? "");
  const [recipientAddress, setRecipientAddress] = useState(initial?.recipient_address ?? "");
  const [jobReference, setJobReference] = useState(initial?.job_reference ?? "");
  const [tone, setTone] = useState(initial?.tone || "formal");
  const [length, setLength] = useState(initial?.length || "medium");
  const [context, setContext] = useState("");
  const [output, setOutput] = useState(initial?.content ?? "");
  const { operation, stopping, start, stop, finish, isCurrent } = useAiOperation();
  const [error, setError] = useState<string | null>(null);
  const [exchanges, setExchanges] = useState<LetterExchange[]>([]);
  const [instructions, setInstructions] = useState<string[]>([]);
  const [instruction, setInstruction] = useState("");
  const [briefOpen, setBriefOpen] = useState(false);
  const [abandonOpen, setAbandonOpen] = useState(false);
  const [overflow, setOverflow] = useState(false);
  const progress = useAiProgress(stopping ? null : (operation?.id ?? null));
  const timer = useAiTimer(operation !== null && !stopping);
  const inIteration = exchanges.length > 0 && !briefOpen;

  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const identity = profile.data?.profile.identity ?? null;

  const saveIdentity = useMutation({
    mutationFn: (next: Identity) => {
      const current = profile.data?.profile;
      if (!current) throw new Error("Profil indisponible");
      return profileService.save({ ...current, identity: next });
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: PROFILE_KEY });
    },
  });

  async function run(nextInstruction: string | null): Promise<void> {
    let id: string;
    try {
      id = start("generation");
    } catch (caught) {
      if (isAiNotConfiguredError(caught)) return;
      setError(message(caught));
      return;
    }
    const suite =
      nextInstruction === null
        ? instructions
        : [...instructions, nextInstruction].slice(-MAX_CONSIGNES);
    setError(null);
    if (nextInstruction === null) setOutput("");
    if (nextInstruction !== null) {
      setInstructions(suite);
      setInstruction("");
      setExchanges((current) => [...current, { auteur: "vous", texte: nextInstruction }]);
    }
    timer.start();
    try {
      const execution = await aiService.generateCoverLetter({
        generation_id: id,
        company: company || null,
        job_title: jobTitle || null,
        tone,
        length,
        context: context || null,
        previous_cover_letter:
          nextInstruction !== null && output.trim().length > 0 ? output : null,
        instruction: suite.length > 0 ? suite.join(" ; ") : null,
      });
      if (!isCurrent(id)) return;
      timer.stop();
      setOutput(execution.output);
      setExchanges((current) => [
        ...current,
        {
          auteur: "candilog",
          texte: formatAiSummary(
            nextInstruction === null ? "Lettre rédigée" : "Lettre régénérée",
            execution.elapsed_ms,
            execution.tokens_used,
          ),
        },
      ]);
      setBriefOpen(false);
    } catch (caught) {
      if (isCurrent(id) && !(caught instanceof AppError && caught.code === "CANCELLED")) {
        setError(message(caught));
        useAiRailStatusStore.getState().setLastOperationFailed(true);
      }
    } finally {
      finish(id);
    }
  }

  async function correct(): Promise<void> {
    const fields = letterCorrectionFields(output);
    if (fields.length === 0) return;
    let id: string;
    try {
      id = start("correction");
    } catch (caught) {
      if (isAiNotConfiguredError(caught)) return;
      setError(message(caught));
      return;
    }
    setError(null);
    timer.start();
    try {
      const execution = await aiService.correctFrench({ generation_id: id, fields });
      if (!isCurrent(id)) return;
      timer.stop();
      const corrected = applyLetterCorrection(output, execution.output.fields);
      setOutput(corrected);
      notify({
        tone: "success",
        title: corrected === output ? "Aucune correction nécessaire" : "Orthographe corrigée",
        detail:
          corrected === output
            ? "Le contenu actuel a été relu sans modification."
            : "La mise en forme, le sens et les faits de la lettre ont été conservés.",
      });
    } catch (caught) {
      if (isCurrent(id) && !(caught instanceof AppError && caught.code === "CANCELLED")) {
        setError(message(caught));
        useAiRailStatusStore.getState().setLastOperationFailed(true);
        notify({ tone: "error", title: "Correction impossible", detail: message(caught) });
      }
    } finally {
      finish(id);
    }
  }

  function letterExport() {
    return {
      name: `Lettre — ${jobTitle || company || "Candidature"}`,
      company: company || null,
      job_title: jobTitle || null,
      recipient: recipient || null,
      recipient_address: recipientAddress || null,
      job_reference: jobReference || null,
      content: output,
    };
  }

  const save = useMutation({
    mutationFn: () => documentsService.saveCoverLetter({ ...letterExport(), tone, length }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: COVER_LETTERS_KEY });
      notify({ tone: "success", title: "Lettre enregistrée" });
    },
    onError: (caught) =>
      notify({ tone: "error", title: "Enregistrement impossible", detail: detail(caught) }),
  });

  async function stopOperation(): Promise<void> {
    try {
      await stop();
    } catch (caught) {
      setError(message(caught));
    }
  }

  function abandon(): void {
    setAbandonOpen(false);
    setOutput("");
    setExchanges([]);
    setInstructions([]);
    setInstruction("");
    setBriefOpen(false);
  }

  return {
    company,
    setCompany,
    jobTitle,
    setJobTitle,
    recipient,
    setRecipient,
    recipientAddress,
    setRecipientAddress,
    jobReference,
    setJobReference,
    tone,
    setTone,
    length,
    setLength,
    context,
    setContext,
    output,
    setOutput,
    operation,
    stopping,
    error,
    exchanges,
    instruction,
    setInstruction,
    briefOpen,
    setBriefOpen,
    abandonOpen,
    setAbandonOpen,
    overflow,
    setOverflow,
    progress,
    elapsedMs: timer.elapsedMs,
    inIteration,
    identity,
    saveIdentity: async (next: Identity): Promise<void> => {
      await saveIdentity.mutateAsync(next);
    },
    isSaving: save.isPending,
    run,
    correct,
    stop: stopOperation,
    save: () => save.mutate(),
    exportPdf: () => exportCoverLetterPdf(letterExport(), notify),
    readClipboard: () => documentsService.readClipboard(),
    abandon,
  };
}
