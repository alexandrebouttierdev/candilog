import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { documentsService } from "../services/documentsService";
import { normalizeResumeWorkspace } from "../model/resumeWorkspace";
import type { CoverLetter, ResumeSummary, ResumeVersion } from "@/shared/types/generated/documents";
import type { ResumeGeneration } from "@/features/ai";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { exportCoverLetterPdf, exportResumePdf } from "./documentExport";
import { COVER_LETTERS_KEY, RESUME_KEY } from "./documentKeys";

/** Onglets de la barre d'outils (`screens/07-resumes.png`). */
export type DocumentFilter = "all" | "resumes" | "letters" | "analyses";

/** Lignes chargées par groupe, puis à chaque « Afficher plus ». */
export const DOCUMENT_STEP = 50;

/** Document ouvert dans l'inspecteur. */
export type DocumentSelection = { kind: "resume"; id: string } | { kind: "letter"; id: string };

function detail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

function isLegacyGeneration(value: unknown): value is ResumeGeneration {
  return typeof value === "object" && value !== null && "resume" in value && "analysis" in value;
}

/**
 * Orchestration de l'écran Documents : CV et lettres groupés, filtres Tous / CV / Lettres /
 * Analyses, et l'inspecteur du document ouvert (export PDF, duplication, copie,
 * suppression). Chaque groupe est une requête bornée côté SQLite, recherche comprise.
 */
export function useDocumentsViewModel(filter: DocumentFilter) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [search, setSearchState] = useState("");
  const searchQuery = useDebounce(search);
  const [limits, setLimits] = useState({ resume: DOCUMENT_STEP, letter: DOCUMENT_STEP });
  const [selected, setSelected] = useState<DocumentSelection | null>(null);
  const [pendingDelete, setPendingDelete] = useState<DocumentSelection | null>(null);

  const showResumes = filter !== "letters";
  const showLetters = filter === "all" || filter === "letters";

  const resumes = useQuery({
    queryKey: [...RESUME_KEY, "bibliotheque", { search: searchQuery, limit: limits.resume, scored: filter === "analyses" }],
    queryFn: () =>
      documentsService.listResumePage({
        page: 1,
        page_size: limits.resume,
        search: searchQuery,
        scored_only: filter === "analyses",
      }),
    enabled: showResumes,
  });
  const letters = useQuery({
    queryKey: [...COVER_LETTERS_KEY, "bibliotheque", { search: searchQuery, limit: limits.letter }],
    queryFn: () => documentsService.listCoverLettersPage({ page: 1, page_size: limits.letter, search: searchQuery }),
    enabled: showLetters,
  });

  const resumeItems: ResumeSummary[] = showResumes ? (resumes.data?.items ?? []) : [];
  const letterItems: CoverLetter[] = showLetters ? (letters.data?.items ?? []) : [];

  // Comme la maquette, l'inspecteur n'est jamais vide : à défaut de sélection encore
  // visible, le premier document de la liste est ouvert.
  const visible =
    selected &&
    (selected.kind === "resume"
      ? resumeItems.some((item) => item.id === selected.id)
      : letterItems.some((item) => item.id === selected.id));
  const current: DocumentSelection | null = visible
    ? selected
    : resumeItems[0]
      ? { kind: "resume", id: resumeItems[0].id }
      : letterItems[0]
        ? { kind: "letter", id: letterItems[0].id }
        : null;

  const resumeDetail = useQuery({
    queryKey: [...RESUME_KEY, current?.kind === "resume" ? current.id : null],
    queryFn: () => documentsService.getResume(current?.id ?? ""),
    enabled: current?.kind === "resume",
  });
  const version: ResumeVersion | null = current?.kind === "resume" ? (resumeDetail.data ?? null) : null;
  const workspace = version ? normalizeResumeWorkspace(version.content) : null;
  const generation = version && isLegacyGeneration(version.content) ? version.content : null;
  const resumeSummary = current?.kind === "resume" ? (resumeItems.find((item) => item.id === current.id) ?? null) : null;
  const letter = current?.kind === "letter" ? (letterItems.find((item) => item.id === current.id) ?? null) : null;

  const invalidate = () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: RESUME_KEY }),
      queryClient.invalidateQueries({ queryKey: COVER_LETTERS_KEY }),
    ]);

  const remove = useMutation({
    mutationFn: (target: DocumentSelection) =>
      target.kind === "resume" ? documentsService.deleteResume(target.id) : documentsService.deleteCoverLetter(target.id),
    onSuccess: async (_result, target) => {
      setSelected(null);
      await invalidate();
      notify({ tone: "success", title: target.kind === "resume" ? "CV supprimé" : "Lettre supprimée" });
    },
    onError: (error) => notify({ tone: "error", title: "Suppression impossible", detail: detail(error) }),
  });

  const duplicate = useMutation({
    mutationFn: (source: ResumeVersion) =>
      documentsService.saveResume({ name: `${source.name} (copie)`, content: source.content }),
    onSuccess: async (copy) => {
      await invalidate();
      setSelected({ kind: "resume", id: copy.id });
      notify({ tone: "success", title: "CV dupliqué", detail: copy.name });
    },
    onError: (error) => notify({ tone: "error", title: "Duplication impossible", detail: detail(error) }),
  });

  /** Export PDF du document ouvert ; une ancienne génération est d'abord mise en page. */
  async function exportPdf(): Promise<void> {
    if (workspace) {
      await exportResumePdf(workspace.document, notify);
      return;
    }
    if (generation) {
      try {
        const prepared = await documentsService.prepareResume(generation);
        await exportResumePdf(prepared.document, notify);
      } catch (error) {
        notify({ tone: "error", title: "Export PDF impossible", detail: detail(error) });
      }
      return;
    }
    if (letter) {
      await exportCoverLetterPdf(
        {
          name: letter.name,
          company: letter.company,
          job_title: letter.job_title,
          recipient: letter.recipient,
          recipient_address: letter.recipient_address,
          job_reference: letter.job_reference,
          content: letter.content,
        },
        notify,
      );
    }
  }

  async function copyLetter(): Promise<void> {
    if (!letter) return;
    await navigator.clipboard.writeText(letter.content);
    notify({ tone: "success", title: "Lettre copiée" });
  }

  return {
    filter,
    search,
    setSearch: (value: string) => {
      setSearchState(value);
      setLimits({ resume: DOCUMENT_STEP, letter: DOCUMENT_STEP });
    },
    showResumes,
    showLetters,
    resumes: resumeItems,
    resumesTotal: showResumes ? (resumes.data?.total ?? 0) : 0,
    letters: letterItems,
    lettersTotal: showLetters ? (letters.data?.total ?? 0) : 0,
    isLoading: (showResumes && resumes.isPending) || (showLetters && letters.isPending),
    error: resumes.error ?? letters.error,
    reload: () => void invalidate(),
    showMore: (kind: "resume" | "letter") =>
      setLimits((value) => ({ ...value, [kind]: value[kind] + DOCUMENT_STEP })),
    current,
    select: setSelected,
    resumeSummary,
    version,
    workspace,
    generation,
    isLoadingVersion: current?.kind === "resume" && resumeDetail.isPending,
    letter,
    exportPdf,
    copyLetter,
    duplicate: () => {
      if (version) duplicate.mutate(version);
    },
    isDuplicating: duplicate.isPending,
    pendingDelete,
    askDelete: setPendingDelete,
    confirmDelete: () => {
      if (pendingDelete) remove.mutate(pendingDelete);
      setPendingDelete(null);
    },
    isDeleting: remove.isPending,
  };
}
