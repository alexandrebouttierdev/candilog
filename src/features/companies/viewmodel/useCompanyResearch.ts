import { useCallback, useState } from "react";
import {
  aiService,
  isAiNotConfiguredError,
  useAiOperation,
  useAiProgress,
  useAiRequiredStore,
} from "@/features/ai";
import { useRemoteSendGuard } from "@/features/settings";
import { useOperationTimer } from "@/shared/hooks/useOperationTimer";
import { AppError } from "@/shared/types/app-error";
import type { CompanyCandidate, CompanyResearchResult } from "@/shared/types/generated/ai";
import {
  companyResearchResultSchema,
  companySearchResultSchema,
} from "../model/schemas/company-research.schema";

/** Étape du dialogue « Compléter avec l'IA ». */
export type CompanyResearchPhase =
  | "closed"
  | "searching"
  | "choosing"
  | "none"
  | "researching"
  | "review"
  | "error";

/** Nature de l'échec, pour un message et une issue adaptés. */
export type CompanyResearchFailure = "network" | "provider" | "unexpected";

/** Ce qui part vers un service distant, dit au premier envoi (décision D4). */
const REMOTE_CONTENT =
  "Le nom de l'entreprise et les informations publiques trouvées (annuaire officiel, site, résultats de recherche)";

function failureOf(error: unknown): { kind: CompanyResearchFailure; message: string } {
  if (error instanceof AppError) {
    if (error.code === "HTTP_ERROR") return { kind: "network", message: error.message };
    return { kind: "provider", message: error.message };
  }
  return {
    kind: "unexpected",
    message: "La réponse reçue est illisible. Réessayez dans un instant.",
  };
}

/**
 * Orchestration de « Compléter avec l'IA » dans le formulaire entreprise.
 *
 * Deux appels : les établissements de l'annuaire (le choix revient à l'utilisateur dès
 * qu'il y en a plusieurs), puis les informations de celui qui est choisi, montrées pour
 * vérification. Rien n'est versé dans le formulaire sans validation explicite.
 */
export function useCompanyResearch() {
  const operation = useAiOperation();
  const confirmSend = useRemoteSendGuard();
  const [phase, setPhase] = useState<CompanyResearchPhase>("closed");
  const [query, setQuery] = useState({ name: "", location: "" });
  const [candidates, setCandidates] = useState<CompanyCandidate[]>([]);
  const [truncated, setTruncated] = useState(false);
  const [selected, setSelected] = useState<CompanyCandidate | null>(null);
  const [result, setResult] = useState<CompanyResearchResult | null>(null);
  const [failure, setFailure] = useState<{ kind: CompanyResearchFailure; message: string } | null>(null);
  const [runningId, setRunningId] = useState<string | null>(null);
  const progress = useAiProgress(runningId);
  const timer = useOperationTimer(runningId !== null);

  /** Démarre une opération IA ; `null` si l'IA n'est pas configurée (popup centrale). */
  const begin = useCallback((): string | null => {
    try {
      const id = operation.start("recherche");
      setRunningId(id);
      timer.start();
      return id;
    } catch (error) {
      if (isAiNotConfiguredError(error)) {
        setPhase("closed");
        return null;
      }
      throw error;
    }
  }, [operation, timer]);

  const end = useCallback(
    (id: string) => {
      operation.finish(id);
      timer.stop();
      setRunningId((current) => (current === id ? null : current));
    },
    [operation, timer],
  );

  const fail = useCallback((error: unknown) => {
    if (error instanceof AppError && error.code === "CANCELLED") {
      setPhase("closed");
      return;
    }
    setFailure(failureOf(error));
    setPhase("error");
  }, []);

  const research = useCallback(
    async (candidate: CompanyCandidate | null, name: string) => {
      setSelected(candidate);
      // Le modèle peut être distant : premier envoi confirmé, comme toute tâche IA.
      if (!(await confirmSend("research_company", REMOTE_CONTENT))) {
        setPhase(candidate ? "choosing" : "none");
        return;
      }
      const id = begin();
      if (!id) return;
      setPhase("researching");
      setFailure(null);
      try {
        const execution = await aiService.researchCompany({
          generation_id: id,
          name,
          siret: candidate?.siret ?? null,
        });
        if (!operation.isCurrent(id)) return;
        setResult(companyResearchResultSchema.parse(execution.output));
        setPhase("review");
      } catch (error) {
        if (operation.isCurrent(id)) fail(error);
      } finally {
        end(id);
      }
    },
    [begin, confirmSend, end, fail, operation],
  );

  const search = useCallback(
    async (name: string, location: string) => {
      const trimmed = name.trim();
      if (trimmed === "") return;
      // Réglages encore inconnus ou IA absente : la popup centrale propose de la configurer.
      if (useAiRequiredStore.getState().configured() !== true) {
        useAiRequiredStore.getState().show();
        return;
      }
      setQuery({ name: trimmed, location });
      setCandidates([]);
      setResult(null);
      setSelected(null);
      setFailure(null);
      const id = begin();
      if (!id) return;
      setPhase("searching");
      let found: CompanyCandidate[] = [];
      let confident = false;
      try {
        const execution = await aiService.searchCompanies({
          generation_id: id,
          name: trimmed,
          location: location.trim() === "" ? null : location.trim(),
        });
        if (!operation.isCurrent(id)) return;
        const parsed = companySearchResultSchema.parse(execution.output);
        found = parsed.candidates;
        confident = parsed.confident;
        setCandidates(found);
        setTruncated(parsed.truncated);
      } catch (error) {
        if (operation.isCurrent(id)) fail(error);
        end(id);
        return;
      }
      end(id);
      const unique = found[0];
      if (found.length === 0) {
        setPhase("none");
      } else if (confident && unique) {
        // Un seul établissement, au nom identique : pas de sélection inutile, la
        // vérification reste montrée avant tout remplissage.
        await research(unique, trimmed);
      } else {
        setSelected(null);
        setPhase("choosing");
      }
    },
    [begin, end, fail, operation, research],
  );

  const close = useCallback(() => {
    if (runningId) void operation.stop().catch(() => undefined);
    setPhase("closed");
    setRunningId(null);
  }, [operation, runningId]);

  return {
    phase,
    open: phase !== "closed",
    query,
    candidates,
    truncated,
    selected,
    result,
    failure,
    progress,
    elapsedMs: timer.elapsedMs,
    running: runningId !== null,
    stopping: operation.stopping,
    search: (name: string, location = "") => void search(name, location),
    choose: setSelected,
    confirmChoice: () => {
      if (selected) void research(selected, query.name);
    },
    webOnly: () => void research(null, query.name),
    retry: () => void search(query.name, query.location),
    backToChoice: () => setPhase(candidates.length > 0 ? "choosing" : "none"),
    configureAi: () => useAiRequiredStore.getState().show(),
    close,
  };
}

export type CompanyResearchViewModel = ReturnType<typeof useCompanyResearch>;
