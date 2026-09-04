import { StrictMode, type PropsWithChildren } from "react";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { aiService, generation_id } from "../../services/aiService";
import { useAiOperationStore } from "../ai-operation-store";
import { useAiOperation } from "../useAiOperation";

vi.mock("../../services/aiService", () => ({
  aiService: { cancel: vi.fn() },
  generation_id: vi.fn(),
}));

function StrictWrapper({ children }: PropsWithChildren) {
  return <StrictMode>{children}</StrictMode>;
}

describe("useAiOperation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useAiOperationStore.setState({ active: null });
    vi.mocked(generation_id).mockReturnValue("gen-1");
    vi.mocked(aiService.cancel).mockResolvedValue(undefined);
  });

  it("démarre et termine uniquement l'opération courante", () => {
    const { result } = renderHook(() => useAiOperation());

    let firstId = "";
    act(() => {
      firstId = result.current.start("analyse");
      result.current.finish(firstId);
    });
    vi.mocked(generation_id).mockReturnValue("gen-2");
    let secondId = "";
    act(() => {
      secondId = result.current.start("generation");
      result.current.finish(firstId);
    });

    expect(firstId).toBe("gen-1");
    expect(secondId).toBe("gen-2");
    expect(result.current.isCurrent(firstId)).toBe(false);
    expect(result.current.isCurrent(secondId)).toBe(true);
    expect(result.current.operation).toMatchObject({ id: "gen-2", kind: "generation" });

    act(() => result.current.finish(secondId));
    expect(result.current.operation).toBeNull();
  });

  it("reconnaît l'opération depuis la fermeture qui vient de la démarrer", () => {
    const { result } = renderHook(() => useAiOperation());
    const isCurrentBeforeStart = result.current.isCurrent;
    let id = "";

    act(() => { id = result.current.start("analyse"); });

    expect(isCurrentBeforeStart(id)).toBe(true);
  });

  it("refuse un second démarrage sans remplacer l'opération active", () => {
    const firstOwner = renderHook(() => useAiOperation());
    const secondOwner = renderHook(() => useAiOperation());
    let firstId = "";
    act(() => { firstId = firstOwner.result.current.start("analyse"); });

    expect(() => {
      act(() => { secondOwner.result.current.start("generation"); });
    }).toThrow("Impossible de démarrer une opération IA : une opération est déjà active.");

    expect(useAiOperationStore.getState().active).toMatchObject({
      id: firstId,
      kind: "analyse",
    });
    expect(generation_id).toHaveBeenCalledOnce();
  });

  it("arrête seulement l'opération possédée par le hook", async () => {
    vi.mocked(aiService.cancel).mockResolvedValue(undefined);
    const firstOwner = renderHook(() => useAiOperation());
    const secondOwner = renderHook(() => useAiOperation());
    let firstId = "";
    act(() => { firstId = firstOwner.result.current.start("analyse"); });

    expect(secondOwner.result.current.isCurrent(firstId)).toBe(false);
    await act(async () => { await secondOwner.result.current.stop(); });

    expect(aiService.cancel).not.toHaveBeenCalled();
    expect(useAiOperationStore.getState().active?.id).toBe(firstId);

    await act(async () => { await firstOwner.result.current.stop(); });
    expect(aiService.cancel).toHaveBeenCalledOnce();
    expect(aiService.cancel).toHaveBeenCalledWith(firstId);
    expect(useAiOperationStore.getState().active).toBeNull();
  });

  it("marque l'opération avant l'appel d'annulation puis la nettoie", async () => {
    let resolveCancel: (() => void) | undefined;
    vi.mocked(aiService.cancel).mockImplementation(
      () => new Promise<void>((resolve) => { resolveCancel = resolve; }),
    );
    const { result } = renderHook(() => useAiOperation());
    act(() => { result.current.start("import"); });

    let stopping: Promise<void> | undefined;
    act(() => { stopping = result.current.stop(); });

    expect(aiService.cancel).toHaveBeenCalledWith("gen-1");
    expect(result.current.stopping).toBe(true);
    expect(result.current.operation?.id).toBe("gen-1");

    await act(async () => {
      resolveCancel?.();
      await stopping;
    });
    expect(result.current.operation).toBeNull();
  });

  it("invalide immédiatement un résultat qui se résout pendant l'annulation", async () => {
    let resolveCancel: (() => void) | undefined;
    vi.mocked(aiService.cancel).mockImplementation(
      () => new Promise<void>((resolve) => { resolveCancel = resolve; }),
    );
    const { result } = renderHook(() => useAiOperation());
    let id = "";
    act(() => { id = result.current.start("generation"); });

    let stopping: Promise<void> | undefined;
    act(() => { stopping = result.current.stop(); });

    expect(result.current.isCurrent(id)).toBe(false);
    expect(result.current.operation).toMatchObject({ id, stopping: true });

    act(() => { result.current.finish(id); });
    expect(result.current.operation).toMatchObject({ id, stopping: true });

    await act(async () => {
      resolveCancel?.();
      await stopping;
    });
    expect(result.current.operation).toBeNull();
  });

  it("nettoie une opération déjà terminée quand l'arrêt échoue", async () => {
    let rejectCancel: ((error: Error) => void) | undefined;
    vi.mocked(aiService.cancel).mockImplementation(
      () => new Promise<void>((_resolve, reject) => { rejectCancel = reject; }),
    );
    const { result } = renderHook(() => useAiOperation());
    let id = "";
    act(() => { id = result.current.start("generation"); });

    let stopping: Promise<void> | undefined;
    act(() => { stopping = result.current.stop(); });
    act(() => { result.current.finish(id); });

    rejectCancel?.(new Error("indisponible"));
    await expect(stopping).rejects.toThrow("indisponible");

    expect(result.current.operation).toBeNull();
  });

  it("restaure l'opération et propage l'échec de l'annulation", async () => {
    vi.mocked(aiService.cancel).mockRejectedValue(new Error("indisponible"));
    const { result } = renderHook(() => useAiOperation());
    act(() => { result.current.start("analyse"); });

    await expect(result.current.stop()).rejects.toThrow("indisponible");

    expect(result.current.operation).toMatchObject({ id: "gen-1", stopping: false });
  });

  it("annule et retire l'opération au démontage sans modifier la suivante", async () => {
    let resolveCancel: (() => void) | undefined;
    vi.mocked(aiService.cancel).mockImplementation(
      () => new Promise<void>((resolve) => { resolveCancel = resolve; }),
    );
    const { result, unmount } = renderHook(() => useAiOperation());
    act(() => { result.current.start("analyse"); });

    unmount();
    expect(aiService.cancel).toHaveBeenCalledWith("gen-1");
    expect(useAiOperationStore.getState().active).toBeNull();

    useAiOperationStore.getState().begin({
      id: "gen-2",
      kind: "generation",
      stop: vi.fn().mockResolvedValue(undefined),
    });
    resolveCancel?.();
    await Promise.resolve();

    expect(useAiOperationStore.getState().active?.id).toBe("gen-2");
  });

  it("reste monté après le cycle de vérification de StrictMode", async () => {
    vi.mocked(aiService.cancel).mockResolvedValue(undefined);
    const { result } = renderHook(() => useAiOperation(), {
      wrapper: StrictWrapper,
      reactStrictMode: true,
    });
    act(() => { result.current.start("analyse"); });

    await act(async () => { await result.current.stop(); });

    expect(aiService.cancel).toHaveBeenCalledWith("gen-1");
    expect(result.current.operation).toBeNull();
  });
});
