import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useUiStore } from "@/shared/lib/ui-store";
import { documentsService } from "../../services/documentsService";
import { workspaceFixture } from "../../model/resumeWorkspace";
import { useBaseResumeViewModel } from "../useBaseResumeViewModel";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const documentExemple = workspaceFixture().document;

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("ViewModel du CV de base", () => {
  it("compose la feuille dès le montage", async () => {
    const compose = vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });

    await waitFor(() => expect(result.current.document).not.toBeNull());

    expect(compose).toHaveBeenCalledWith([]);
  });

  it("recompose sans la section retirée", async () => {
    const compose = vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });
    await waitFor(() => expect(result.current.document).not.toBeNull());

    act(() => result.current.toggle("skills"));

    await waitFor(() => expect(compose).toHaveBeenLastCalledWith(["skills"]));
  });

  it("demande confirmation avant de recomposer une feuille retouchée", async () => {
    vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });
    await waitFor(() => expect(result.current.document).not.toBeNull());

    act(() => result.current.setDocument({ ...documentExemple, profile: "retouché" }));
    act(() => result.current.toggle("skills"));

    expect(result.current.pendingToggle).toBe("skills");
    expect(result.current.excluded).toEqual([]);

    act(() => result.current.confirmToggle());

    await waitFor(() => expect(result.current.excluded).toEqual(["skills"]));
    expect(result.current.pendingToggle).toBeNull();
  });

  it("annule la bascule en attente sans toucher aux sections retenues", async () => {
    vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });
    await waitFor(() => expect(result.current.document).not.toBeNull());

    act(() => result.current.setDocument({ ...documentExemple, profile: "retouché" }));
    act(() => result.current.toggle("skills"));
    expect(result.current.pendingToggle).toBe("skills");

    act(() => result.current.cancelToggle());

    expect(result.current.pendingToggle).toBeNull();
    expect(result.current.excluded).toEqual([]);
  });

  it("enregistre le document sous la forme « CV de base »", async () => {
    vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const saveResume = vi.spyOn(documentsService, "saveResume").mockResolvedValue({
      id: "resume-1",
      name: "CV de base",
      content: documentExemple,
      created_at: "2026-10-10T00:00:00Z",
    });
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });
    await waitFor(() => expect(result.current.document).not.toBeNull());

    await act(async () => { await result.current.save(); });

    expect(saveResume).toHaveBeenCalledWith({
      name: "CV de base",
      content: { schema_version: 1, kind: "base", document: documentExemple },
      version_note: "Composé depuis le profil",
    });
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("CV ajouté à la bibliothèque");
  });
});
