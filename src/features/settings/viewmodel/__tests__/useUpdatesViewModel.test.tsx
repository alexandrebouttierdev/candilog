import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { UpdateInfo, UpdateProgress } from "@/shared/types/generated/settings";
import { settingsService } from "../../services/settingsService";
import { useUpdatesViewModel } from "../useUpdatesViewModel";

const unlisten = vi.fn();
let progressHandler: ((event: { payload: UpdateProgress }) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_name: string, handler: typeof progressHandler) => {
    progressHandler = handler;
    return Promise.resolve(unlisten);
  }),
}));

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const update: UpdateInfo = {
  version: "1.2.0",
  notes: "Corrections",
  page_url: "https://github.com/alexandrebouttierdev/candilog/releases/tag/v1.2.0",
  asset: { name: "candilog.deb", url: "https://github.com/asset" },
};

beforeEach(() => {
  vi.restoreAllMocks();
  unlisten.mockReset();
  progressHandler = null;
  vi.spyOn(settingsService, "about").mockResolvedValue({ version: "1.1.0", name: "Candilog" });
});

describe("ViewModel des mises à jour", () => {
  it("charge la version et expose le résultat d'une vérification", async () => {
    vi.spyOn(settingsService, "checkUpdate").mockResolvedValue(update);
    const { result } = renderHook(() => useUpdatesViewModel(), { wrapper });
    await waitFor(() => expect(result.current.version).toBe("1.1.0"));

    await act(async () => { await result.current.check(); });

    expect(result.current.update).toEqual(update);
    expect(result.current.error).toBeNull();
  });

  it("écoute la progression et retire le listener au démontage", async () => {
    const { result, unmount } = renderHook(() => useUpdatesViewModel(), { wrapper });
    await waitFor(() => expect(progressHandler).not.toBeNull());

    act(() => progressHandler?.({ payload: { progress: 42 } }));
    expect(result.current.progress).toBe(42);
    unmount();
    expect(unlisten).toHaveBeenCalledOnce();
  });

  it("ouvre la page officielle quand aucun installateur n'est publié", async () => {
    const withoutAsset = { ...update, asset: null };
    vi.spyOn(settingsService, "checkUpdate").mockResolvedValue(withoutAsset);
    const openReleasePage = vi.spyOn(settingsService, "openReleasePage").mockResolvedValue(undefined);
    const downloadUpdate = vi.spyOn(settingsService, "downloadUpdate");
    const { result } = renderHook(() => useUpdatesViewModel(), { wrapper });
    await act(async () => { await result.current.check(); });

    await act(async () => { await result.current.download(); });

    expect(openReleasePage).toHaveBeenCalledWith(withoutAsset.page_url);
    expect(downloadUpdate).not.toHaveBeenCalled();
  });

  it("affiche une erreur de vérification lisible", async () => {
    vi.spyOn(settingsService, "checkUpdate").mockRejectedValue(new Error("hors ligne"));
    const { result } = renderHook(() => useUpdatesViewModel(), { wrapper });

    await act(async () => { await result.current.check().catch(() => undefined); });

    expect(result.current.error).toBe("Vérification impossible.");
  });
});
