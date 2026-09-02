import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useUiStore } from "@/shared/lib/ui-store";
import { AppError } from "@/shared/types/app-error";
import { analyticsService } from "../../services/analyticsService";
import { useAnalyticsViewModel } from "../useAnalyticsViewModel";

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
  vi.spyOn(analyticsService, "load").mockImplementation(() => new Promise(() => undefined));
});

describe("export des analyses", () => {
  it("exporte la période courante et annonce le succès", async () => {
    const exportCsv = vi.spyOn(analyticsService, "exportCsv").mockResolvedValue(true);
    const { result } = renderHook(() => useAnalyticsViewModel(), { wrapper });

    await act(async () => { await result.current.exportCsv(); });

    expect(exportCsv).toHaveBeenCalledWith("trente_days");
    expect(useUiStore.getState().toasts.at(-1)?.title).toBe("Analyses exportées");
  });

  it("reste silencieux si le sélecteur d'export est annulé", async () => {
    vi.spyOn(analyticsService, "exportCsv").mockResolvedValue(false);
    const { result } = renderHook(() => useAnalyticsViewModel(), { wrapper });

    await act(async () => { await result.current.exportCsv(); });

    expect(useUiStore.getState().toasts).toHaveLength(0);
  });

  it("reprend le message applicatif d'un échec", async () => {
    vi.spyOn(analyticsService, "exportCsv").mockRejectedValue(
      new AppError({ code: "VALIDATION_ERROR", message: "Destination inaccessible." }),
    );
    const { result } = renderHook(() => useAnalyticsViewModel(), { wrapper });

    await act(async () => { await result.current.exportCsv().catch(() => undefined); });

    expect(useUiStore.getState().toasts.at(-1)).toMatchObject({
      tone: "error",
      title: "Export impossible",
      detail: "Destination inaccessible.",
    });
  });
});
