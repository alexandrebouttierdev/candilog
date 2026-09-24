import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ManagedModelStatus, ManagedOllamaStatus } from "@/shared/types/generated/ai";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { managedOllamaService } from "../../../services/managedOllamaService";
import type { ManagedOllamaViewModel } from "../../../viewmodel/useManagedOllamaViewModel";
import { LocalInstallOverlay } from "../LocalInstallOverlay";

function modele(id: ManagedModelStatus["definition"]["id"], name: string, overrides: Partial<ManagedModelStatus> = {}): ManagedModelStatus {
  return {
    definition: {
      id,
      category: "light",
      publisher: "mistral",
      publisher_label: "Mistral AI",
      display_name: name,
      description: "",
      ollama_tag: `${id}:tag`,
      approximate_download_bytes: 2_100_000_000,
      recommended_ram_gb: 4,
    },
    installed: false,
    active: false,
    machine_fit: "compatible",
    recommended: false,
    last_benchmark: null,
    ...overrides,
  };
}

const STATUS: ManagedOllamaStatus = {
  runtime_state: "not_installed",
  runtime_version: null,
  port: null,
  models_disk_bytes: 0,
  active_model: null,
  models: [
    modele("ministral3_light", "Ministral 3 · 3B", { recommended: true }),
    modele("ministral3_balanced", "Ministral 3 · 8B"),
    modele("mistral_small_quality", "Mistral Small", { machine_fit: "insufficient_memory" }),
  ],
  last_error: null,
};

function viewModel(installAsync: ManagedOllamaViewModel["installAsync"]): ManagedOllamaViewModel {
  return {
    runtimeState: "not_installed",
    status: STATUS,
    progress: null,
    error: null,
    isInstalling: false,
    isRemoving: false,
    isActivating: false,
    install: vi.fn(),
    installAsync,
    cancel: vi.fn(),
    remove: vi.fn(),
    activate: vi.fn(),
    activateAsync: vi.fn(),
    reload: vi.fn(),
  };
}

beforeEach(() => {
  vi.restoreAllMocks();
  useUiStore.setState({ toasts: [] });
});

describe("installer l'IA locale", () => {
  it("préselectionne le modèle recommandé et écarte celui que la machine ne porte pas", () => {
    render(<LocalInstallOverlay vm={viewModel(vi.fn())} onClose={vi.fn()} />);

    expect(screen.getByRole("radio", { name: /Ministral 3 · 3B/ })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: /Mistral Small/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Installer · 2,1 Go" })).toBeEnabled();
  });

  it("installe le modèle choisi, envoie la phrase de test et annonce la fin en place", async () => {
    const installAsync = vi.fn().mockResolvedValue(STATUS);
    vi.spyOn(managedOllamaService, "probe").mockResolvedValue({ model: "ministral3_balanced:tag", latency_ms: 240 });
    render(<LocalInstallOverlay vm={viewModel(installAsync)} onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("radio", { name: /Ministral 3 · 8B/ }));
    await userEvent.click(screen.getByRole("button", { name: "Installer · 2,1 Go" }));

    expect(await screen.findByRole("heading", { name: "Installation terminée" })).toBeInTheDocument();
    expect(installAsync).toHaveBeenCalledWith("ministral3_balanced");
    expect(screen.getByText("répondue en 240 ms")).toBeInTheDocument();
    expect(screen.getByText("Ministral 3 · 8B est prêt")).toBeInTheDocument();
    // Le succès est visible : pas de toast en plus.
    expect(useUiStore.getState().toasts).toHaveLength(0);
  });

  it("propose de reprendre une installation annulée", async () => {
    const installAsync = vi.fn().mockRejectedValue(new AppError({ code: "CANCELLED", message: "annulé" }));
    render(<LocalInstallOverlay vm={viewModel(installAsync)} onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Installer · 2,1 Go" }));

    const alerte = await screen.findByRole("alert");
    expect(alerte).toHaveTextContent("Installation annulée");
    await userEvent.click(within(alerte).getByRole("button", { name: "Reprendre l’installation" }));
    expect(installAsync).toHaveBeenCalledTimes(2);
  });

  it("dit pourquoi l'installation a échoué", async () => {
    const installAsync = vi.fn().mockRejectedValue(new AppError({ code: "PROVIDER_ERROR", message: "Espace disque insuffisant." }));
    render(<LocalInstallOverlay vm={viewModel(installAsync)} onClose={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Installer · 2,1 Go" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Espace disque insuffisant.");
  });
});
