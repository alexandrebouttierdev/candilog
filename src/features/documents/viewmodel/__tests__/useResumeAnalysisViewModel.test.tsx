import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { aiService } from "@/features/ai/services/aiService";
import { useAiOperationStore } from "@/features/ai/viewmodel/ai-operation-store";
import { AppError } from "@/shared/types/app-error";
import { useResumeAnalysisViewModel } from "../useResumeAnalysisViewModel";

vi.mock("@/features/ai/viewmodel/useAiProgress", () => ({
  useAiProgress: () => null,
}));

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useAiOperationStore.setState({ active: null });
});

describe("ViewModel de l'analyse de CV", () => {
  it("refuse l'analyse sans fichier", async () => {
    const analyze = vi.spyOn(aiService, "analyzeResume");
    const { result } = renderHook(() => useResumeAnalysisViewModel(), { wrapper });
    await act(async () => { await result.current.run(); });
    expect(result.current.error).toBe("Choisissez le CV PDF à analyser.");
    expect(analyze).not.toHaveBeenCalled();
  });

  it("analyse le fichier retenu côté natif", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({ name: "cv.pdf" });
    const analyze = vi.spyOn(aiService, "analyzeResume").mockResolvedValue({
      output: {
        resume: { resume: "", experiences: [], skills: [], education: [] },
        job_offer: { title: "Dev", skills: [], soft_skills: [], experience: null, keywords: [] },
        score: { total: 72, skills: null, experience: null, ats: null, present: [], missing: [] },
        analysis: { recap: "Correct", recommendations: [], content_recommendations: [] },
      },
      elapsed_ms: 18_400,
      tokens_used: 1_024,
    });
    const { result } = renderHook(() => useResumeAnalysisViewModel(), { wrapper });
    act(() => result.current.setJobOffer("Une offre"));
    await act(async () => { await result.current.selectFile(); });
    await act(async () => { await result.current.run(); });
    expect(analyze.mock.calls[0]?.[0]).toMatchObject({ job_offer: "Une offre" });
    expect(result.current.result?.analysis.recap).toBe("Correct");
    expect(result.current.metrics).toEqual({ elapsed_ms: 18_400, tokens_used: 1_024 });
  });

  it("conserve le fichier après une erreur fournisseur", async () => {
    vi.spyOn(aiService, "selectResumeFile").mockResolvedValue({ name: "cv.pdf" });
    vi.spyOn(aiService, "analyzeResume").mockRejectedValue(
      new AppError({ code: "PROVIDER_ERROR", message: "Le fournisseur ne répond pas." }),
    );
    const { result } = renderHook(() => useResumeAnalysisViewModel(), { wrapper });
    act(() => result.current.setJobOffer("Une offre"));
    await act(async () => { await result.current.selectFile(); });
    await act(async () => { await result.current.run(); });
    expect(result.current.error).toBe("Le fournisseur ne répond pas.");
    expect(result.current.selectedFile?.name).toBe("cv.pdf");
  });
});
