import type { ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AppError } from "@/shared/types/app-error";
import type {
  LocalAiDownloadError,
  LocalAiRecommendation,
  LocalAiStatus,
} from "@/shared/types/generated/ai";
import { useUiStore } from "@/shared/lib/ui-store";
import { localAiService } from "../../services/localAiService";
import { useLocalAiViewModel } from "../useLocalAiViewModel";

const unlisten = vi.fn();
let errorHandler: ((event: LocalAiDownloadError) => void) | null = null;

vi.mock("../../services/localAiService", () => ({
  localAiService: {
    recommendation: vi.fn(),
    status: vi.fn(),
    install: vi.fn(),
    cancel: vi.fn(),
    remove: vi.fn(),
    benchmark: vi.fn(),
    test: vi.fn(),
    onProgress: vi.fn(() => Promise.resolve(unlisten)),
    onCompleted: vi.fn(() => Promise.resolve(unlisten)),
    onError: vi.fn((handler: (event: LocalAiDownloadError) => void) => {
      errorHandler = handler;
      return Promise.resolve(unlisten);
    }),
  },
}));

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

const idleStatus: LocalAiStatus = {
  state: "not_configured",
  active_model: null,
  installed_models: [],
  backend: null,
  benchmark: null,
  last_error: null,
};

const recommendation = {
  selected_model: null,
  backend: "cpu",
  evaluations: [],
  reason: "",
  hardware: {},
} as unknown as LocalAiRecommendation;

beforeEach(() => {
  vi.clearAllMocks();
  errorHandler = null;
  useUiStore.setState({ toasts: [] });
  vi.mocked(localAiService.recommendation).mockResolvedValue(recommendation);
  vi.mocked(localAiService.status).mockResolvedValue(idleStatus);
});

describe("ViewModel de l'IA locale", () => {
  it("traite l'annulation d'installation comme un retour à idle, pas une erreur", async () => {
    vi.mocked(localAiService.install).mockRejectedValue(
      new AppError({ code: "CANCELLED", message: "Génération annulée." }),
    );
    const { result } = renderHook(() => useLocalAiViewModel(), { wrapper });
    await waitFor(() => expect(result.current.state).toBe("not_configured"));

    act(() => {
      result.current.install("ministral3_light");
    });

    await waitFor(() => {
      expect(result.current.error).toBeNull();
      expect(result.current.state).toBe("not_configured");
      expect(result.current.progress).toBeNull();
    });
    const titles = useUiStore.getState().toasts.map((toast) => toast.title);
    expect(titles).toContain("Téléchargement annulé");
    expect(titles.some((title) => title.includes("impossible") || title.includes("annulée"))).toBe(
      false,
    );
  });

  it("ignore un événement download-error CANCELLED sans ErrorBanner", async () => {
    const { result } = renderHook(() => useLocalAiViewModel(), { wrapper });
    await waitFor(() => expect(errorHandler).not.toBeNull());

    act(() => {
      errorHandler?.({
        model_id: "ministral3_light",
        code: "CANCELLED",
        message: "Génération annulée.",
      });
    });

    expect(result.current.error).toBeNull();
    expect(result.current.state).not.toBe("error");
    await waitFor(() => {
      expect(result.current.state).toBe("not_configured");
    });
    expect(
      useUiStore.getState().toasts.some((toast) => toast.title === "Téléchargement annulé"),
    ).toBe(true);
  });

  it("conserve l'erreur pour un vrai échec d'installation", async () => {
    vi.mocked(localAiService.install).mockRejectedValue(
      new AppError({ code: "PROVIDER_ERROR", message: "Le réseau est indisponible." }),
    );
    const { result } = renderHook(() => useLocalAiViewModel(), { wrapper });
    await waitFor(() => expect(result.current.state).toBe("not_configured"));

    act(() => {
      result.current.install("ministral3_light");
    });
    await waitFor(() => expect(result.current.state).toBe("error"));

    expect(result.current.error).toBe("Le réseau est indisponible.");
  });
});
