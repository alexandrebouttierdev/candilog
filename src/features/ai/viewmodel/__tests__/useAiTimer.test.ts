import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAiTimer } from "../useAiTimer";

describe("useAiTimer", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("ne montre aucune durée avant le premier traitement", () => {
    const { result } = renderHook(() => useAiTimer(false));

    expect(result.current.durationMs).toBeNull();
    expect(result.current.elapsedMs).toBe(0);
  });

  it("fige la durée du traitement à l'arrêt", () => {
    const now = vi.spyOn(Date, "now").mockReturnValue(1_000);
    const { result } = renderHook(() => useAiTimer(true));

    act(() => result.current.start());
    now.mockReturnValue(9_500);
    act(() => result.current.stop());

    expect(result.current.durationMs).toBe(8_500);
  });

  it("efface la durée précédente au démarrage suivant", () => {
    const now = vi.spyOn(Date, "now").mockReturnValue(1_000);
    const { result } = renderHook(() => useAiTimer(true));

    act(() => result.current.start());
    now.mockReturnValue(3_000);
    act(() => result.current.stop());
    act(() => result.current.start());

    expect(result.current.durationMs).toBeNull();
  });
});
