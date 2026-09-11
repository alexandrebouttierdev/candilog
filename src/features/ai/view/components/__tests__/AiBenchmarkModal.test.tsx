import { render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AiBenchmarkModal } from "../AiBenchmarkModal";

const runUserBenchmark = vi.fn<() => Promise<never>>();
const cancelAiOperation = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);

vi.mock("../../../viewmodel/importProfile", () => ({
  runUserBenchmark: (generationId: string) => runUserBenchmark(generationId),
  cancelAiOperation: (generationId: string) => cancelAiOperation(generationId),
}));

describe("AiBenchmarkModal", () => {
  afterEach(() => {
    vi.clearAllMocks();
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
});
