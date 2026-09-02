import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { settingsService } from "../../services/settingsService";
import { useBackupsViewModel } from "../useBackupsViewModel";

function setup() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return { client, ...renderHook(() => useBackupsViewModel(), { wrapper }) };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("ViewModel des sauvegardes", () => {
  it("ignore silencieusement un export annulé puis annonce un export terminé", async () => {
    const exportBackup = vi.spyOn(settingsService, "export").mockResolvedValueOnce(false).mockResolvedValueOnce(true);
    const { result } = setup();

    await act(async () => { await result.current.exportBackup(); });
    expect(useUiStore.getState().toasts).toHaveLength(0);
    await act(async () => { await result.current.exportBackup(); });

    expect(exportBackup).toHaveBeenCalledTimes(2);
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Sauvegarde créée");
  });

  it("restaure, invalide les données et referme la confirmation", async () => {
    vi.spyOn(settingsService, "restore").mockResolvedValue(true);
    const { client, result } = setup();
    const invalidate = vi.spyOn(client, "invalidateQueries");
    act(() => result.current.openRestore());

    await act(async () => { await result.current.restoreBackup(); });

    expect(invalidate).toHaveBeenCalledOnce();
    expect(result.current.restoreOpen).toBe(false);
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Sauvegarde restaurée");
  });

  it("referme une restauration annulée sans invalider les données", async () => {
    vi.spyOn(settingsService, "restore").mockResolvedValue(false);
    const { client, result } = setup();
    const invalidate = vi.spyOn(client, "invalidateQueries");
    act(() => result.current.openRestore());

    await act(async () => { await result.current.restoreBackup(); });

    expect(invalidate).not.toHaveBeenCalled();
    expect(result.current.restoreOpen).toBe(false);
  });

  it("affiche le détail applicatif d'une erreur", async () => {
    vi.spyOn(settingsService, "export").mockRejectedValue(
      new AppError({ code: "DATABASE_ERROR", message: "Le disque est indisponible." }),
    );
    const { result } = setup();

    await act(async () => { await result.current.exportBackup().catch(() => undefined); });

    expect(useUiStore.getState().toasts.at(-1)).toMatchObject({
      tone: "error",
      title: "Export impossible",
      detail: "Le disque est indisponible.",
    });
  });

  it("invalide les données après une réinitialisation complète", async () => {
    vi.spyOn(settingsService, "reset").mockResolvedValue({ data_cleared: true, secret_cleared: true });
    const { client, result } = setup();
    const invalidate = vi.spyOn(client, "invalidateQueries");
    act(() => result.current.openReset());

    await act(async () => { await result.current.resetData(); });

    expect(invalidate).toHaveBeenCalledOnce();
    expect(result.current.resetOpen).toBe(false);
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Données et clé API réinitialisées");
  });
});
