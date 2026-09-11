import { act, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AiBenchmarkModal } from "../AiBenchmarkModal";
import type { UserBenchmarkResult } from "@/shared/types/generated/ai";

const runUserBenchmark = vi.fn<(generationId: string) => Promise<UserBenchmarkResult>>();
const cancelAiOperation = vi.fn<(generationId: string) => Promise<void>>().mockResolvedValue(undefined);

vi.mock("../../../viewmodel/importProfile", () => ({
  runUserBenchmark: (generationId: string) => runUserBenchmark(generationId),
  cancelAiOperation: (generationId: string) => cancelAiOperation(generationId),
}));

function sampleResult(overrides: Partial<UserBenchmarkResult> = {}): UserBenchmarkResult {
  return {
    score: 78,
    quality: "good",
    metrics: {
      total_ms: 75_000,
      pdf_extract_ms: 1_200,
      preprocess_ms: 800,
      llm_ms: 70_000,
      parse_ms: 400,
      llm_calls: 2,
      tokens_input: 1_200,
      tokens_output: 450,
      tokens_per_second: 12.5,
    },
    categories: [
      { label: "Identité", score: 10, max_score: 10 },
      { label: "Expériences", score: 20, max_score: 30 },
    ],
    hallucination_count: 0,
    benchmark_version: 1,
    provider_label: "IA locale",
    model_label: "ministral-3:3b",
    remote_warning: false,
    method_used: "vision",
    fallback_used: false,
    ...overrides,
  };
}

describe("AiBenchmarkModal", () => {
  afterEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
  });

  it("ne relance pas le benchmark quand onClose change d'identité", async () => {
    runUserBenchmark.mockImplementation(() => new Promise(() => undefined));

    const { rerender } = render(
      <AiBenchmarkModal open onClose={() => undefined} modelLabel="ministral-3:3b" />,
    );

    await waitFor(() => expect(runUserBenchmark).toHaveBeenCalledTimes(1));

    // Même pattern que AiPage / AiGlobalHeader : callback inline recréé à chaque rendu.
    rerender(<AiBenchmarkModal open onClose={() => undefined} modelLabel="ministral-3:3b" />);

    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(runUserBenchmark).toHaveBeenCalledTimes(1);
    expect(cancelAiOperation).not.toHaveBeenCalled();
  });

  it("affiche une barre de progression indéterminée pendant le test", async () => {
    runUserBenchmark.mockImplementation(() => new Promise(() => undefined));

    render(<AiBenchmarkModal open onClose={() => undefined} modelLabel="ministral-3:3b" />);

    await waitFor(() => {
      expect(screen.getByRole("status")).toHaveTextContent("Analyse du CV de référence…");
    });
    // ModalHost porte le contenu hors du conteneur RTL (portail sur document.body).
    expect(document.querySelector(".import-indeterminate")).not.toBeNull();
    expect(screen.getByText(/Candilog envoie un CV de test/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Arrêter" })).toBeInTheDocument();
  });

  it("convertit les durées en minutes au-delà de 60 s et détaille le résultat", async () => {
    runUserBenchmark.mockResolvedValue(sampleResult());

    render(<AiBenchmarkModal open onClose={() => undefined} modelLabel="ministral-3:3b" />);

    await waitFor(() => {
      expect(screen.getByText("78")).toBeInTheDocument();
    });
    expect(screen.getByText("Bon")).toBeInTheDocument();
    expect(screen.getByText("1 min 15 s")).toBeInTheDocument();
    expect(screen.getByText("1 min 10 s")).toBeInTheDocument();
    expect(screen.getByText(/IA locale/)).toBeInTheDocument();
    expect(screen.getByText(/Vision/)).toBeInTheDocument();
    expect(screen.getByText("Aucune")).toBeInTheDocument();
    expect(screen.getByText("Identité")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Retester" })).toBeInTheDocument();
  });

  it("affiche le chronomètre mm:ss pendant l'attente", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    runUserBenchmark.mockImplementation(() => new Promise(() => undefined));

    render(<AiBenchmarkModal open onClose={() => undefined} modelLabel="ministral-3:3b" />);

    await waitFor(() => {
      expect(screen.getByRole("status")).toBeInTheDocument();
    });

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1_200);
    });

    expect(screen.getByRole("status").textContent).toMatch(/00:0[1-2]/);
  });
});
