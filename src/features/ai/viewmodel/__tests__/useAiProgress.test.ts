import { renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useCancelAiOnUnmount } from "../useAiProgress";
import { aiService } from "../../services/aiService";

describe("useCancelAiOnUnmount", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(aiService, "cancel").mockResolvedValue(undefined);
  });

  it("n'annule pas quand l'opération se termine sans quitter l'écran", () => {
    const initialProps: { generation_id: string | null } = { generation_id: "op-1" };
    const { rerender } = renderHook(
      ({ generation_id }: { generation_id: string | null }) =>
        useCancelAiOnUnmount(generation_id),
      { initialProps },
    );

    rerender({ generation_id: null });

    expect(aiService.cancel).not.toHaveBeenCalled();
  });

  it("annule au démontage si une génération est encore en cours", () => {
    const { unmount } = renderHook(() => useCancelAiOnUnmount("op-1"));

    unmount();

    expect(aiService.cancel).toHaveBeenCalledWith("op-1");
  });
});
