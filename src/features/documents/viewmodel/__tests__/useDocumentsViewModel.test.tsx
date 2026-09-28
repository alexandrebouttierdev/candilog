import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { documentsService } from "../../services/documentsService";
import { useUiStore } from "@/shared/lib/ui-store";
import { workspaceFixture } from "../../model/resumeWorkspace";
import { useDocumentsViewModel } from "../useDocumentsViewModel";
import type { CoverLetter } from "@/shared/types/generated/documents";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const LETTRE: CoverLetter = {
  id: "l-1",
  name: "Lettre Astek",
  company: "Astek",
  job_title: "Dev",
  recipient: "Service recrutement",
  recipient_address: null,
  job_reference: "FS-114",
  tone: "formal",
  length: "medium",
  content: "Madame, Monsieur,",
  created_at: "2026-08-30T00:00:00Z",
};

function pageOf<T>(items: T[]) {
  return { items, total: items.length, page: 1, page_size: 50, total_pages: 1 };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("ViewModel de la bibliothèque de documents", () => {
  it("charge les CV et le détail du premier, ouvert par défaut", async () => {
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue(
      pageOf([{ id: "cv-1", name: "CV Produit", created_at: "2026-08-30T00:00:00Z", ats_score: 82, target_title: "Dev" }]),
    );
    vi.spyOn(documentsService, "listCoverLettersPage").mockResolvedValue(pageOf([LETTRE]));
    const getResume = vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-1",
      name: "CV Produit",
      content: workspaceFixture(),
      created_at: "2026-08-30T00:00:00Z",
    });

    const { result } = renderHook(() => useDocumentsViewModel("all"), { wrapper });

    await waitFor(() => expect(result.current.resumes).toHaveLength(1));
    expect(result.current.letters).toHaveLength(1);
    expect(result.current.current).toEqual({ kind: "resume", id: "cv-1" });
    await waitFor(() => expect(getResume).toHaveBeenCalledWith("cv-1"));
    await waitFor(() => expect(result.current.workspace).not.toBeNull());
  });

  it("exporte un workspace via le service PDF", async () => {
    const workspace = workspaceFixture();
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue(
      pageOf([{ id: "cv-1", name: "CV Produit", created_at: "2026-08-30T00:00:00Z", ats_score: null, target_title: null }]),
    );
    vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-1",
      name: "CV Produit",
      content: workspace,
      created_at: "2026-08-30T00:00:00Z",
    });
    const exportPdf = vi.spyOn(documentsService, "exportPdf").mockResolvedValue(true);

    const { result } = renderHook(() => useDocumentsViewModel("resumes"), { wrapper });
    await waitFor(() => expect(result.current.workspace).not.toBeNull());

    await act(async () => {
      await result.current.exportPdf();
    });

    expect(exportPdf).toHaveBeenCalledWith(workspace.document);
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("CV exporté");
  });

  it("exporte une lettre avec ses mentions, sans charger les CV", async () => {
    const listResumes = vi.spyOn(documentsService, "listResumePage");
    vi.spyOn(documentsService, "listCoverLettersPage").mockResolvedValue(pageOf([LETTRE]));
    const exportLetter = vi.spyOn(documentsService, "exportCoverLetterPdf").mockResolvedValue(true);

    const { result } = renderHook(() => useDocumentsViewModel("letters"), { wrapper });
    await waitFor(() => expect(result.current.letter?.id).toBe("l-1"));

    await act(async () => {
      await result.current.exportPdf();
    });

    expect(listResumes).not.toHaveBeenCalled();
    expect(exportLetter).toHaveBeenCalledWith(
      expect.objectContaining({ name: "Lettre Astek", recipient: "Service recrutement", job_reference: "FS-114" }),
    );
  });

  it("supprime le document confirmé et l'annonce", async () => {
    vi.spyOn(documentsService, "listCoverLettersPage").mockResolvedValue(pageOf([LETTRE]));
    const remove = vi.spyOn(documentsService, "deleteCoverLetter").mockResolvedValue(undefined);

    const { result } = renderHook(() => useDocumentsViewModel("letters"), { wrapper });
    await waitFor(() => expect(result.current.letter).not.toBeNull());

    act(() => result.current.askDelete({ kind: "letter", id: "l-1" }));
    act(() => result.current.confirmDelete());

    await waitFor(() => expect(remove).toHaveBeenCalledWith("l-1"));
    await waitFor(() => expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Lettre supprimée"));
  });
});
