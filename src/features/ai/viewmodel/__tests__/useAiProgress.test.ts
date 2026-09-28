import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { AiProgress } from "../../model/types";
import { useAiProgress } from "../useAiProgress";

let handler: ((event: { payload: AiProgress }) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_name: string, next: typeof handler) => {
    handler = next;
    return Promise.resolve(vi.fn());
  }),
}));

function progress(generation_id: string, step: string): AiProgress {
  return { generation_id, step, chunk: null, tokens_used: null };
}

describe("progression d'une génération", () => {
  it("retient la première étape annoncée avant que l'opération soit connue", async () => {
    const { result, rerender } = renderHook(({ id }) => useAiProgress(id), {
      initialProps: { id: null as string | null },
    });
    await act(async () => {
      await Promise.resolve();
    });

    // Le backend annonce l'étape dès le lancement, avant que l'écran ait l'identifiant.
    act(() => handler?.({ payload: progress("gen-1", "Rédaction") }));
    rerender({ id: "gen-1" });

    expect(result.current?.step).toBe("Rédaction");
  });

  it("ignore la progression d'une autre génération", async () => {
    const { result } = renderHook(() => useAiProgress("gen-1"));
    await act(async () => {
      await Promise.resolve();
    });

    act(() => handler?.({ payload: progress("autre", "Rédaction") }));

    expect(result.current).toBeNull();
  });
});
