import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { documentsService } from "../../services/documentsService";
import { useUiStore } from "@/shared/lib/ui-store";
import { workspaceFixture } from "../../model/resumeWorkspace";
import { useResumeLibraryViewModel } from "../useResumeLibraryViewModel";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("ViewModel de la bibliothèque de CV", () => {
  it("charge la page et le détail de la version sélectionnée", async () => {
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-1", name: "CV Produit", created_at: "2026-08-30T00:00:00Z" }],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    const getResume = vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-1",
      name: "CV Produit",
      content: workspaceFixture(),
      created_at: "2026-08-30T00:00:00Z",
    });

    const { result } = renderHook(() => useResumeLibraryViewModel(), { wrapper });

    await waitFor(() => expect(result.current.versions).toHaveLength(1));
    await waitFor(() => expect(getResume).toHaveBeenCalledWith("cv-1"));
    expect(result.current.workspace).not.toBeNull();
  });

  it("exporte un workspace via le service PDF", async () => {
    const workspace = workspaceFixture();
    vi.spyOn(documentsService, "listResumePage").mockResolvedValue({
      items: [{ id: "cv-1", name: "CV Produit", created_at: "2026-08-30T00:00:00Z" }],
      total: 1,
      page: 1,
      page_size: 8,
      total_pages: 1,
    });
    vi.spyOn(documentsService, "getResume").mockResolvedValue({
      id: "cv-1",
      name: "CV Produit",
      content: workspace,
      created_at: "2026-08-30T00:00:00Z",
    });
    const exportPdf = vi.spyOn(documentsService, "exportPdf").mockResolvedValue(true);

    const { result } = renderHook(() => useResumeLibraryViewModel(), { wrapper });
    await waitFor(() => expect(result.current.workspace).not.toBeNull());

    await act(async () => {
      await result.current.exportWorkspacePdf();
    });

    expect(exportPdf).toHaveBeenCalledWith(workspace.document);
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("CV exporté");
  });
});
