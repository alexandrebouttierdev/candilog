import { renderHook, act } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useElapsedClock } from "../useElapsedClock";

describe("useElapsedClock", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-08-29T16:00:00"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("avance chaque seconde tant que l'analyse tourne", () => {
    const startedAt = Date.now();
    const { result } = renderHook(() => useElapsedClock(true, startedAt));

    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(result.current).toBe(3000);
  });

  it("arrête le timer quand l'analyse se termine", () => {
    const startedAt = Date.now();
    const { result, rerender } = renderHook(
      ({ running }: { running: boolean }) => useElapsedClock(running, startedAt),
      { initialProps: { running: true } },
    );

    act(() => {
      vi.advanceTimersByTime(2000);
    });
    rerender({ running: false });
    act(() => {
      vi.advanceTimersByTime(5000);
    });

    expect(result.current).toBe(2000);
  });

  it("nettoie l'intervalle au démontage", () => {
    const startedAt = Date.now();
    const { unmount } = renderHook(() => useElapsedClock(true, startedAt));
    unmount();
    expect(vi.getTimerCount()).toBe(0);
  });
});
