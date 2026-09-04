import { beforeEach, describe, expect, it, vi } from "vitest";
import { useAiOperationStore } from "../ai-operation-store";

describe("ai operation store", () => {
  beforeEach(() => {
    useAiOperationStore.setState({ active: null });
  });

  it("ignore la fin et le changement d'état d'une opération obsolète", () => {
    const stop = vi.fn().mockResolvedValue(undefined);
    const state = useAiOperationStore.getState();

    state.begin({ id: "old", kind: "analyse", stop });
    state.begin({ id: "current", kind: "generation", stop });
    state.markStopping("old", true);
    state.finish("old");

    expect(useAiOperationStore.getState().active).toMatchObject({
      id: "current",
      kind: "generation",
      stopping: false,
    });
  });
});
