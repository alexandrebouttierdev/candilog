import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { documentsService } from "../services/documentsService";
import { PROFILE_KEY, profileService } from "@/features/profile";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { PAGE_SIZE } from "@/shared/types/page";
import { useDebounce } from "@/shared/hooks/useDebounce";
import { exportCoverLetterPdf } from "./documentExport";
import { COVER_LETTERS_KEY } from "./documentKeys";

function detail(error: unknown): string | undefined {
  return error instanceof AppError ? error.message : undefined;
}

/** Bibliothèque paginée des lettres enregistrées. */
export function useLettersLibraryViewModel() {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [selected, setSelected] = useState<string | null>(null);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [search, setSearch] = useState("");
  const searchQuery = useDebounce(search);

  const list = useQuery({
    queryKey: [...COVER_LETTERS_KEY, "page", { page, search: searchQuery }],
    queryFn: () =>
      documentsService.listCoverLettersPage({
        page,
        page_size: PAGE_SIZE,
        search: searchQuery,
      }),
  });

  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const identity = profile.data?.profile.identity ?? null;

  const coverLetters = list.data?.items ?? [];
  const selectedId = coverLetters.some((letter) => letter.id === selected)
    ? selected
    : (coverLetters[0]?.id ?? null);
  const selectedLetter = coverLetters.find((letter) => letter.id === selectedId) ?? null;

  const remove = useMutation({
    mutationFn: documentsService.deleteCoverLetter,
    onSuccess: async () => {
      setSelected(null);
      setDeleteId(null);
      await queryClient.invalidateQueries({ queryKey: COVER_LETTERS_KEY });
    },
    onError: (error) => {
      setDeleteId(null);
      notify({ tone: "error", title: "Suppression impossible", detail: detail(error) });
    },
  });

  async function copySelected(): Promise<void> {
    if (!selectedLetter) return;
    await navigator.clipboard.writeText(selectedLetter.content);
    notify({ tone: "success", title: "Lettre copiée" });
  }

  async function exportSelectedPdf(): Promise<void> {
    if (!selectedLetter) return;
    await exportCoverLetterPdf(
      {
        name: selectedLetter.name,
        company: selectedLetter.company,
        job_title: selectedLetter.job_title,
        recipient: selectedLetter.recipient,
        recipient_address: selectedLetter.recipient_address,
        job_reference: selectedLetter.job_reference,
        content: selectedLetter.content,
      },
      notify,
    );
  }

  function updateSearch(value: string): void {
    setSearch(value);
    setPage(1);
  }

  return {
    page, setPage, search, updateSearch, list, coverLetters, selectedId, setSelected,
    selectedLetter, deleteId, setDeleteId, identity, isDeleting: remove.isPending,
    remove: remove.mutate, copySelected, exportSelectedPdf, pageSize: PAGE_SIZE,
  };
}
