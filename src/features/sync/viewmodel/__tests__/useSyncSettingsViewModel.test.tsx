import type { ReactNode } from "react";
import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createTestQueryClient } from "@/shared/lib/test-utils";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import type { RemoteProbe, SyncStatus } from "@/shared/types/generated/sync";
import { syncService } from "../../services/syncService";
import { useSyncSettingsViewModel } from "../useSyncSettingsViewModel";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => undefined)),
}));

const status = (provider: SyncStatus["provider"]): SyncStatus => ({
  provider,
  phase: provider === "local" ? "not_configured" : "up_to_date",
  auto_sync: true,
  location: provider === "local" ? null : "/tmp/espace",
  pending_changes: 0,
  conflicts: 0,
  waiting_changes: 0,
  missing_files: 0,
  computers: 1,
  last_success_at: null,
  last_attempt_at: null,
  next_attempt_at: null,
  message: null,
});

const probe: RemoteProbe = {
  provider: "folder",
  location: "/tmp/espace",
  space_exists: false,
  legacy_space_exists: false,
  space_created_at: null,
  local_has_data: true,
};

function rendu() {
  const client = createTestQueryClient();
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return renderHook(() => useSyncSettingsViewModel(), { wrapper });
}

const toasts = () => useUiStore.getState().toasts;

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(syncService, "availability").mockResolvedValue({ google_drive: false, one_drive: false });
});

describe("réglages de synchronisation", () => {
  it("ne lit pas les conflits tant que l'ordinateur est en mode local", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("local"));
    const conflicts = vi.spyOn(syncService, "conflicts").mockResolvedValue([]);
    const { result } = rendu();

    await waitFor(() => expect(result.current.status?.provider).toBe("local"));
    expect(conflicts).not.toHaveBeenCalled();
    expect(result.current.conflicts).toEqual([]);
  });

  it("lit les conflits dès qu'un espace est rattaché", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("folder"));
    const conflicts = vi.spyOn(syncService, "conflicts").mockResolvedValue([]);
    rendu();

    await waitFor(() => expect(conflicts).toHaveBeenCalled());
  });

  it("ouvre la mise en service après le choix d'un dossier, rien si le dialogue est fermé", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("local"));
    const choose = vi.spyOn(syncService, "chooseFolder").mockResolvedValueOnce(null).mockResolvedValueOnce(probe);
    const { result } = rendu();

    act(() => result.current.chooseFolder());
    await waitFor(() => expect(choose).toHaveBeenCalledTimes(1));
    expect(result.current.probe).toBeNull();

    act(() => result.current.chooseFolder());
    await waitFor(() => expect(result.current.probe).toEqual(probe));
  });

  it("se tait quand l'utilisateur annule la connexion à un service", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("local"));
    vi.spyOn(syncService, "connect").mockRejectedValue(new AppError({ code: "CANCELLED", message: "Connexion abandonnée." }));
    const { result } = rendu();

    act(() => result.current.connect("google_drive"));
    await waitFor(() => expect(result.current.connecting).toBeNull());
    expect(toasts()).toEqual([]);
  });

  it("signale une connexion refusée", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("local"));
    vi.spyOn(syncService, "connect").mockRejectedValue(new AppError({ code: "SYNC_ERROR", message: "Accès refusé." }));
    const { result } = rendu();

    act(() => result.current.connect("one_drive"));
    await waitFor(() => expect(toasts()).toHaveLength(1));
    expect(toasts()[0]).toMatchObject({ tone: "error", title: "Connexion impossible", detail: "Accès refusé." });
  });

  it("annonce la sauvegarde prise avant de remplacer les données locales", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("local"));
    vi.spyOn(syncService, "chooseFolder").mockResolvedValue(probe);
    vi.spyOn(syncService, "setup").mockResolvedValue({ backup_file: "candilog-avant-synchronisation.sqlite" });
    const { result } = rendu();

    act(() => result.current.chooseFolder());
    await waitFor(() => expect(result.current.probe).not.toBeNull());
    await act(async () => {
      await result.current.setup({ choice: "join_replace_local" });
    });

    expect(result.current.probe).toBeNull();
    expect(toasts()[0]).toMatchObject({
      tone: "success",
      title: "Synchronisation activée",
      detail: expect.stringContaining("candilog-avant-synchronisation.sqlite") as string,
    });
  });

  it("referme la confirmation après la désactivation", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("folder"));
    vi.spyOn(syncService, "conflicts").mockResolvedValue([]);
    vi.spyOn(syncService, "disconnect").mockResolvedValue(undefined);
    const { result } = rendu();

    act(() => result.current.openDialog("disconnect"));
    expect(result.current.dialog).toBe("disconnect");
    act(() => result.current.disconnect());

    await waitFor(() => expect(result.current.dialog).toBeNull());
    expect(toasts()[0]).toMatchObject({ tone: "success", title: "Synchronisation désactivée" });
  });

  it("accorde le décompte des conflits résolus", async () => {
    vi.spyOn(syncService, "status").mockResolvedValue(status("folder"));
    vi.spyOn(syncService, "conflicts").mockResolvedValue([]);
    vi.spyOn(syncService, "resolveAll").mockResolvedValueOnce(1).mockResolvedValueOnce(3);
    const { result } = rendu();

    act(() => result.current.resolveAll("keep_local"));
    await waitFor(() => expect(toasts()[0]?.title).toBe("1 conflit résolu"));
    act(() => result.current.resolveAll("keep_local"));
    await waitFor(() => expect(toasts()[0]?.title).toBe("3 conflits résolus"));
  });
});
