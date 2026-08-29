import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useProfileImportProgress } from "../useProfileImportProgress";

const unlisten = vi.fn();
let handler: ((event: { payload: { generation_id: string; at: string; message: string; step: string | null } }) => void) | null =
  null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_name: string, next: typeof handler) => {
    handler = next;
    return Promise.resolve(unlisten);
  }),
}));

describe("useProfileImportProgress", () => {
  beforeEach(() => {
    handler = null;
    unlisten.mockReset();
  });

  it("ajoute les lignes du journal et ignore un autre identifiant", async () => {
    const { result } = renderHook(() => useProfileImportProgress("gen-1"));
    await act(async () => {
      await Promise.resolve();
    });

    act(() => {
      handler?.({
        payload: {
          generation_id: "gen-1",
          at: "2026-08-29T14:32:01.000Z",
          message: "Lecture du fichier",
          step: "Lecture du fichier…",
        },
      });
      handler?.({
        payload: {
          generation_id: "autre",
          at: "2026-08-29T14:32:02.000Z",
          message: "ignoré",
          step: null,
        },
      });
    });

    expect(result.current.step).toBe("Lecture du fichier…");
    expect(result.current.entries).toHaveLength(1);
    expect(result.current.entries[0]?.message).toBe("Lecture du fichier");
  });

  it("retire le listener au démontage", async () => {
    const { unmount } = renderHook(() => useProfileImportProgress("gen-1"));
    await act(async () => {
      await Promise.resolve();
    });
    unmount();
    expect(unlisten).toHaveBeenCalledOnce();
  });
});
