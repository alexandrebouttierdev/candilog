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

  it("demande confirmation avant de recomposer une feuille retouchée, puis recompose réellement", async () => {
    // Un document distinct par appel (reflétant les sections écartées reçues) : sans cela,
    // la garde par référence de la copie locale ne distinguerait jamais une recomposition
    // d'une réponse déjà appliquée, et le test ne prouverait que l'intention, pas l'effet.
    const compose = vi
      .spyOn(documentsService, "composeBaseResume")
      .mockImplementation((excluded = []) => Promise.resolve({ ...documentExemple, profile: `profil:${excluded.join(",")}` }));
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });
    await waitFor(() => expect(result.current.document).not.toBeNull());
    expect(result.current.document?.profile).toBe("profil:");

    act(() => result.current.setDocument({ ...documentExemple, profile: "retouché" }));
    act(() => result.current.toggle("skills"));

    expect(result.current.pendingToggle).toBe("skills");
    expect(result.current.excluded).toEqual([]);
    // Rien n'est recomposé avant confirmation : la retouche est toujours là.
    expect(result.current.document?.profile).toBe("retouché");

    act(() => result.current.confirmToggle());

    await waitFor(() => expect(result.current.excluded).toEqual(["skills"]));
    expect(result.current.pendingToggle).toBeNull();
    // La feuille affichée est bien la nouvelle composition, pas la retouche écrasée.
    await waitFor(() => expect(result.current.document?.profile).toBe("profil:skills"));
    expect(compose).toHaveBeenLastCalledWith(["skills"]);
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

  it("enregistre les sections écartées avec le document", async () => {
    vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const saveResume = vi.spyOn(documentsService, "saveResume").mockResolvedValue({
      id: "resume-1",
      name: "CV de base",
      content: documentExemple,
      created_at: "2026-10-10T00:00:00Z",
    });
    const { result } = renderHook(() => useBaseResumeViewModel(), { wrapper });
    await waitFor(() => expect(result.current.document).not.toBeNull());

    act(() => result.current.toggle("skills"));
    await waitFor(() => expect(result.current.excluded).toEqual(["skills"]));
    await act(async () => { await result.current.save(); });

    // Sans ce champ, la réouverture rallumerait « Compétences » et la recomposition
    // suivante les ramènerait sur la feuille.
    expect(saveResume.mock.calls[0]?.[0].content).toMatchObject({ excluded_sections: ["skills"] });
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
      content: { schema_version: 1, kind: "base", document: documentExemple, excluded_sections: [] },
      version_note: "Composée depuis le profil",
    });
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("CV ajouté à la bibliothèque");
  });
});

describe("réouverture d'un CV de base enregistré", () => {
  it("affiche le document rouvert sans le recomposer depuis le profil", () => {
    const compose = vi.spyOn(documentsService, "composeBaseResume");
    const reopened = { ...documentExemple, profile: "Retouché après enregistrement." };
    const { result } = renderHook(
      () => useBaseResumeViewModel({ document: reopened, name: "Ma version retouchée", documentId: "cv-base-1" }),
      { wrapper },
    );

    // Aucune composition : écraser silencieusement la retouche enregistrée serait la perte
    // de travail que cette tâche doit éviter.
    expect(compose).not.toHaveBeenCalled();
    expect(result.current.document).toEqual(reopened);
    expect(result.current.name).toBe("Ma version retouchée");
  });

  it("repart des sections écartées enregistrées, sans ressusciter celles qui l'étaient", async () => {
    const compose = vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const reopened = { ...documentExemple, profile: "Retouché après enregistrement." };
    const { result } = renderHook(
      () =>
        useBaseResumeViewModel({
          document: reopened,
          name: "Ma version retouchée",
          documentId: "cv-base-1",
          excludedSections: ["skills"],
        }),
      { wrapper },
    );

    // L'interrupteur « Compétences » reste éteint : le document rouvert n'en a pas.
    expect(result.current.excluded).toEqual(["skills"]);

    act(() => result.current.toggle("projects"));
    act(() => result.current.confirmToggle());

    // La recomposition retire les projets **et** garde les compétences écartées.
    await waitFor(() => expect(compose).toHaveBeenCalledWith(["skills", "projects"]));
  });

  it("demande confirmation avant de recomposer la première bascule après réouverture", async () => {
    const compose = vi.spyOn(documentsService, "composeBaseResume").mockResolvedValue(documentExemple);
    const reopened = { ...documentExemple, profile: "Retouché après enregistrement." };
    const { result } = renderHook(
      () => useBaseResumeViewModel({ document: reopened, name: "Ma version retouchée", documentId: "cv-base-1" }),
      { wrapper },
    );

    act(() => result.current.toggle("skills"));

    // La bascule attend la confirmation : le document rouvert n'est pas encore écrasé.
    expect(result.current.pendingToggle).toBe("skills");
    expect(compose).not.toHaveBeenCalled();
    expect(result.current.document).toEqual(reopened);

    act(() => result.current.confirmToggle());

    await waitFor(() => expect(compose).toHaveBeenCalledWith(["skills"]));
    await waitFor(() => expect(result.current.document).toEqual(documentExemple));
  });

  it("enregistre une nouvelle version du document rouvert plutôt qu'un second CV de base", async () => {
    const reopened = { ...documentExemple, profile: "Retouché après enregistrement." };
    const saveResume = vi.spyOn(documentsService, "saveResume").mockResolvedValue({
      id: "cv-base-1",
      name: "Ma version retouchée",
      content: reopened,
      created_at: "2026-10-10T00:00:00Z",
    });
    const { result } = renderHook(
      () => useBaseResumeViewModel({ document: reopened, name: "Ma version retouchée", documentId: "cv-base-1" }),
      { wrapper },
    );

    await act(async () => { await result.current.save(); });

    expect(saveResume).toHaveBeenCalledWith({
      name: "Ma version retouchée",
      content: { schema_version: 1, kind: "base", document: reopened, excluded_sections: [] },
      revises: "cv-base-1",
      version_note: "Modifiée depuis le CV de base",
    });
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Nouvelle version enregistrée");
  });

  it("enregistre une version du CV rouvert, puis de la version qu'il vient d'enregistrer", async () => {
    const reopened = { ...documentExemple, profile: "Retouché après enregistrement." };
    const saveResume = vi
      .spyOn(documentsService, "saveResume")
      .mockResolvedValueOnce({ id: "cv-base-2", name: "Ma version retouchée", content: reopened, created_at: "2026-10-10T00:00:00Z" })
      .mockResolvedValueOnce({ id: "cv-base-3", name: "Ma version retouchée", content: reopened, created_at: "2026-10-10T00:01:00Z" });
    const { result } = renderHook(
      () => useBaseResumeViewModel({ document: reopened, name: "Ma version retouchée", documentId: "cv-base-1" }),
      { wrapper },
    );

    await act(async () => { await result.current.save(); });
    await act(async () => { await result.current.save(); });

    // Le second ⌘S révise la version que le premier vient d'enregistrer (`cv-base-2`), pas
    // le document initialement rouvert (`cv-base-1`) : sans `setRevises(saved.id)`, ce
    // second appel porterait encore `revises: "cv-base-1"` et cette assertion échouerait.
    expect(saveResume.mock.calls.map(([input]) => input.revises)).toEqual(["cv-base-1", "cv-base-2"]);
    expect(saveResume.mock.calls[0]?.[0].version_note).toBe("Modifiée depuis le CV de base");
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Nouvelle version enregistrée");
  });
});
