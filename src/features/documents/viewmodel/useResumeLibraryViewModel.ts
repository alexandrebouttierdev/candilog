import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { documentsService } from "../services/documentsService";
import type { ResumeVersion } from "@/shared/types/generated/documents";
import { normalizeResumeWorkspace } from "../model/resumeWorkspace";
import type { ResumeGeneration } from "@/features/ai";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { PAGE_SIZE } from "@/shared/types/page";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { exportResumePdf } from "./documentExport";
import { RESUME_KEY } from "./documentKeys";

function detail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

function isLegacyGeneration(value: unknown): value is ResumeGeneration {
  return typeof value === "object" && value !== null && "resume" in value && "analysis" in value;
}

/** Bibliothèque paginée des CV enregistrés : liste, détail, duplication et export. */
export function useResumeLibraryViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const searchQuery = useDebounce(search);
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);

  const list = useQuery({
    queryKey: [...RESUME_KEY, "page", { page, search: searchQuery }],
    queryFn: () =>
      documentsService.listResumePage({ page, page_size: PAGE_SIZE, search: searchQuery }),
  });

  const versions = list.data?.items ?? [];
  const selectedId = versions.some((resume) => resume.id === selected)
    ? selected
    : (versions[0]?.id ?? null);

  const detailQuery = useQuery({
    queryKey: [...RESUME_KEY, selectedId],
    queryFn: () => documentsService.getResume(selectedId ?? ""),
    enabled: selectedId !== null,
  });

  const remove = useMutation({
    mutationFn: documentsService.deleteResume,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "Version supprimée" });
    },
    onError: (error) => {
      setDeleteId(null);
      notify({ tone: "error", title: "Suppression impossible", detail: detail(error) });
    },
  });

  const duplicate = useMutation({
    mutationFn: async (version: ResumeVersion) => {
      await documentsService.saveResume({
        name: `${version.name} (copie)`,
        content: version.content,
      });
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: RESUME_KEY });
      notify({ tone: "success", title: "Version dupliquée" });
    },
    onError: (error) =>
      notify({ tone: "error", title: "Duplication impossible", detail: detail(error) }),
  });

  const version = detailQuery.data;
  const workspace = version ? normalizeResumeWorkspace(version.content) : null;
  const generation = version && isLegacyGeneration(version.content) ? version.content : null;
  const atsScore = workspace?.score.total ?? generation?.profile_score.total;

  async function exportWorkspacePdf(): Promise<void> {
    if (!workspace) return;
    await exportResumePdf(workspace.document, notify);
  }

  async function exportLegacyPdf(): Promise<void> {
    if (!generation) return;
    try {
      const prepared = await documentsService.prepareResume(generation);
      await exportResumePdf(prepared.document, notify);
    } catch (error) {
      notify({ tone: "error", title: "Export PDF impossible", detail: detail(error) });
    }
  }

  function updateSearch(value: string): void {
    setSearch(value);
    setPage(1);
  }

  function duplicateSelected(): void {
    if (detailQuery.data) duplicate.mutate(detailQuery.data);
  }

  return {
    page, setPage, search, updateSearch, list, versions, selectedId, setSelected,
    deleteId, setDeleteId, version, workspace, generation, atsScore, detail: detailQuery,
    isDeleting: remove.isPending, isDuplicating: duplicate.isPending,
    remove: remove.mutate, duplicate: duplicateSelected,
    exportWorkspacePdf, exportLegacyPdf, pageSize: PAGE_SIZE,
  };
}
