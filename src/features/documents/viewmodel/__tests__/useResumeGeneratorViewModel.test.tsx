import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { aiService } from "@/features/ai/services/aiService";
import type { ResumeGeneration } from "@/features/ai/model/types";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { documentsService } from "../../services/documentsService";
import { workspaceFixture } from "../../model/resumeWorkspace";
import { useResumeGeneratorViewModel } from "../useResumeGeneratorViewModel";
import { useAiOperationStore } from "@/features/ai/viewmodel/ai-operation-store";

vi.mock("@/features/ai/viewmodel/useAiProgress", () => ({
  useAiProgress: () => null,
}));

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

function generation(): ResumeGeneration {
  return {
    resume: { resume: "Profil ciblé", experiences: [], skills: [], education: [] },
    analysis: { recap: "", recommendations: [], content_recommendations: [] },
    job_offer: { title: "Développeur Rust", skills: [], soft_skills: [], experience: null, keywords: [] },
    profile_score: { total: 70, skills: null, experience: null, ats: null, present: [], missing: [] },
    recommendation_error: null,
  };
}

function execution(output = generation()) {
  return { output, elapsed_ms: 18_400, tokens_used: 1_024 };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useAiOperationStore.setState({ active: null });
  useUiStore.setState({ toasts: [] });
});

describe("ViewModel du générateur de CV", () => {
  it("refuse une offre vide sans appeler le fournisseur", async () => {
    const generateResume = vi.spyOn(aiService, "generateResume");
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: null, workspace: null, name: "" }),
      { wrapper },
    );

    await act(async () => { await result.current.generate(); });

    expect(result.current.error).toBe("Collez le texte de l’offre à cibler.");
    expect(generateResume).not.toHaveBeenCalled();
  });

  it("génère puis prépare un workspace autonome", async () => {
    const source = generation();
    const prepared = workspaceFixture();
    vi.spyOn(aiService, "generateResume").mockResolvedValue(execution(source));
    vi.spyOn(documentsService, "prepareResume").mockResolvedValue(prepared);
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: null, workspace: null, name: "" }),
      { wrapper },
    );
    act(() => result.current.setJobOffer("Une offre Rust"));

    await act(async () => { await result.current.generate(); });

    expect(result.current.workspace).toBe(prepared);
    expect(result.current.name).toBe("CV — Développeur");
    expect(result.current.generationIndex).toBe(1);
    expect(result.current.briefOpen).toBe(false);
    expect(result.current.metrics).toEqual({ elapsed_ms: 18_400, tokens_used: 1_024 });
  });

  it("arrête la génération et ignore sa réponse tardive", async () => {
    let resolveGeneration: ((value: ReturnType<typeof execution>) => void) | undefined;
    vi.spyOn(aiService, "generateResume").mockReturnValue(
      new Promise((resolve) => { resolveGeneration = resolve; }),
    );
    const prepareResume = vi.spyOn(documentsService, "prepareResume");
    const cancel = vi.spyOn(aiService, "cancel").mockResolvedValue(undefined);
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: null, workspace: null, name: "" }),
      { wrapper },
    );
    act(() => result.current.setJobOffer("Une offre Rust"));

    let generationPromise: Promise<void> | undefined;
    act(() => { generationPromise = result.current.generate(); });
    await waitFor(() => expect(result.current.operation).not.toBeNull());
    const operationId = result.current.operation?.id;

    await act(async () => { await result.current.stop(); });

    expect(cancel).toHaveBeenCalledWith(operationId);
    expect(result.current.operation).toBeNull();

    await act(async () => {
      resolveGeneration?.(execution());
      await generationPromise;
    });
    expect(prepareResume).not.toHaveBeenCalled();
    expect(result.current.workspace).toBeNull();
    expect(result.current.error).toBeNull();
  });

  it("affiche l'échec de l'arrêt sans produire de rejet non géré", async () => {
    vi.spyOn(aiService, "generateResume").mockReturnValue(new Promise(() => undefined));
    vi.spyOn(aiService, "cancel").mockRejectedValue(
      new AppError({ code: "IO_ERROR", message: "L'arrêt a échoué." }),
    );
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: null, workspace: null, name: "" }),
      { wrapper },
    );
    act(() => result.current.setJobOffer("Une offre Rust"));
    act(() => { void result.current.generate(); });
    await waitFor(() => expect(result.current.operation).not.toBeNull());

    await act(async () => { await result.current.stop(); });

    expect(result.current.error).toBe("L'arrêt a échoué.");
    expect(result.current.operation).toMatchObject({ stopping: false });
  });

  it("prépare une génération historique reçue par navigation", async () => {
    const prepared = workspaceFixture({ profile: "Historique préparé" });
    vi.spyOn(documentsService, "prepareResume").mockResolvedValue(prepared);
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: generation(), workspace: null, name: "CV historique" }),
      { wrapper },
    );

    await waitFor(() => expect(result.current.workspace).toBe(prepared));
    expect(result.current.generationIndex).toBe(1);
  });

  it("affiche le message du fournisseur en cas d'échec", async () => {
    vi.spyOn(aiService, "generateResume").mockRejectedValue(
      new AppError({ code: "PROVIDER_ERROR", message: "Le fournisseur ne répond pas." }),
    );
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: null, workspace: null, name: "" }),
      { wrapper },
    );
    act(() => result.current.setJobOffer("Une offre"));

    await act(async () => { await result.current.generate(); });

    expect(result.current.error).toBe("Le fournisseur ne répond pas.");
    expect(result.current.operation).toBeNull();
  });

  it("enregistre le workspace édité via le ViewModel", async () => {
    const workspace = workspaceFixture();
    const saveResume = vi.spyOn(documentsService, "saveResume").mockResolvedValue({
      id: "resume-1",
      name: "CV Produit",
      content: workspace,
      created_at: "2026-09-02T00:00:00Z",
    });
    const { result } = renderHook(
      () => useResumeGeneratorViewModel({ result: null, workspace, name: "CV Produit" }),
      { wrapper },
    );

    await act(async () => { await result.current.saveResume(workspace); });

    expect(saveResume).toHaveBeenCalledWith({ name: "CV Produit", content: workspace });
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("CV ajouté à la bibliothèque");
  });
});
