import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type {
  LocalAiRecommendation,
  LocalAiStatus,
  LocalModelDefinition,
  LocalModelProfile,
} from "@/shared/types/generated/ai";
import { MistralLocalPanel } from "../MistralLocalPanel";
import { useLocalAiViewModel } from "../../../viewmodel/useLocalAiViewModel";

vi.mock("../../../viewmodel/useLocalAiViewModel", () => ({
  useLocalAiViewModel: vi.fn(),
}));

const actions = {
  install: vi.fn(),
  cancel: vi.fn(),
  remove: vi.fn(),
  benchmark: vi.fn(),
  test: vi.fn(),
  reevaluate: vi.fn(),
};

function model(profile: LocalModelProfile): LocalModelDefinition {
  const rank = profile === "light" ? "3" : profile === "balanced" ? "8" : "14";
  return {
    id: profile === "light" ? "ministral3_light" : profile === "balanced" ? "ministral3_balanced" : "ministral3_quality",
    profile,
    family: "mistral",
    display_name: `Ministral 3 ${rank}B Instruct`,
    repository: `mistralai/test-${rank}`,
    filename: `Ministral-${rank}-Q4_K_M.gguf`,
    local_filename: `ministral-${rank}b-q4_k_m.gguf`,
    revision: "a".repeat(40),
    sha256: "b".repeat(64),
    download_size_bytes: Number(rank) * 500_000_000,
    estimated_ram_mb: Number(rank) * 700,
    recommended_ram_mb: Number(rank) * 1_500,
    recommended_vram_mb: Number(rank) * 1_000,
    context_size: 8192,
    quantization: "Q4_K_M",
    runtime: "llama.cpp",
  };
}

function recommendation(selected: LocalModelDefinition | null): LocalAiRecommendation {
  return {
    hardware: {
      os: "macos",
      architecture: "aarch64",
      cpu: "Apple",
      cpu_model: "Apple M1",
      logical_cores: 8,
      physical_cores: 8,
      total_ram_mb: 16_384,
      available_ram_mb: 12_000,
      gpus: [],
      apple_silicon: true,
      soc_model: "Apple M1",
      unified_memory_mb: 16_384,
      metal: true,
      cuda: false,
      vulkan: false,
    },
    selected_model: selected,
    backend: "metal",
    evaluations: [],
    reason: selected ? "Compatible" : "Mémoire insuffisante",
  };
}

function status(active: LocalModelDefinition | null): LocalAiStatus {
  return {
    state: active ? "ready" : "not_configured",
    active_model: active,
    installed_models: active ? [active] : [],
    backend: active ? "metal" : null,
    benchmark: active
      ? {
          load_time_ms: 1200,
          tokens_per_second: 18.4,
          memory_used_mb: 5100,
          generated_tokens: 64,
          rating: "good",
          measured_at: "2026-09-06T12:00:00Z",
        }
      : null,
    last_error: null,
  };
}

function setup(overrides: Record<string, unknown> = {}) {
  vi.mocked(useLocalAiViewModel).mockReturnValue({
    state: "not_configured",
    recommendation: null,
    status: status(null),
    progress: null,
    error: null,
    testResult: null,
    isRemoving: false,
    isTesting: false,
    ...actions,
    ...overrides,
  });
}

describe("configuration Mistral Local", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setup();
  });

  it("affiche l'état non configuré sans inventer de recommandation", () => {
    render(<MistralLocalPanel />);
    expect(screen.getByText("Fonctionne directement sur votre ordinateur")).toBeInTheDocument();
    expect(document.querySelector("[data-local-ai-state='not_configured']")).toBeInTheDocument();
  });

  it("affiche la détection matérielle", () => {
    setup({ state: "detecting_hardware" });
    render(<MistralLocalPanel />);
    expect(screen.getByRole("status", { name: "Détection de la configuration" })).toBeInTheDocument();
  });

  it.each([
    ["light", "Léger"],
    ["balanced", "Équilibré"],
    ["quality", "Qualité"],
  ] as const)("affiche la recommandation %s", (profile, label) => {
    const selected = model(profile);
    setup({ state: "recommendation_ready", recommendation: recommendation(selected) });
    render(<MistralLocalPanel />);
    expect(screen.getAllByText(label).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "Installer l'IA locale" })).toBeInTheDocument();
  });

  it("refuse proprement une machine incompatible", () => {
    setup({ state: "recommendation_ready", recommendation: recommendation(null) });
    render(<MistralLocalPanel />);
    expect(screen.getByText("IA locale non recommandée")).toBeInTheDocument();
  });

  it("affiche une progression de téléchargement réelle", () => {
    setup({
      state: "downloading",
      recommendation: recommendation(model("balanced")),
      progress: {
        model_id: "ministral3_balanced",
        state: "downloading",
        downloaded_bytes: 3_900_000_000,
        total_bytes: 5_100_000_000,
        bytes_per_second: 42_000_000,
        progress: 76,
      },
    });
    render(<MistralLocalPanel />);
    expect(screen.getByText("76 %")).toBeInTheDocument();
    expect(screen.getByText(/42 Mo\/s/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Annuler" })).toBeInTheDocument();
  });

  it.each([
    ["verifying", "Vérification du téléchargement"],
    ["installing", "Installation du modèle"],
    ["benchmarking", "Mesure des performances"],
  ] as const)("affiche l'étape %s", (stateValue, title) => {
    setup({ state: stateValue, recommendation: recommendation(model("light")) });
    render(<MistralLocalPanel />);
    expect(screen.getByText(title)).toBeInTheDocument();
  });

  it("affiche une erreur de téléchargement", () => {
    setup({ state: "error", error: "Le réseau est indisponible." });
    render(<MistralLocalPanel />);
    expect(screen.getByText("Le réseau est indisponible.")).toBeInTheDocument();
  });

  it("affiche le modèle prêt, le benchmark et le test", async () => {
    const active = model("balanced");
    setup({ state: "ready", recommendation: recommendation(active), status: status(active), testResult: "Notre équipe d'assistance locale répond." });
    render(<MistralLocalPanel />);
    expect(screen.getByText("IA locale prête")).toBeInTheDocument();
    expect(screen.getByText("18,4 tokens/s")).toBeInTheDocument();
    // La prose du modèle n'est pas un message d'état : interrogé sur « l'assistance locale »,
    // il répondait « Notre équipe d'assistance locale est entièrement opérationnelle… ».
    // Seul le fait que le test ait abouti est affiché.
    expect(screen.getByText("Votre modèle local est installé et opérationnel.")).toBeInTheDocument();
    expect(screen.queryByText("Notre équipe d'assistance locale répond.")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Tester l'IA" }));
    expect(actions.test).toHaveBeenCalledOnce();
  });

  it("propose le profil inférieur après un benchmark trop lent sans le télécharger", async () => {
    const active = model("quality");
    const lower = model("balanced");
    const slowStatus = status(active);
    if (slowStatus.benchmark) slowStatus.benchmark.rating = "too_slow";
    setup({
      state: "ready",
      recommendation: { ...recommendation(active), evaluations: [{ model: lower, compatibility: "supported", reason: "Compatible" }] },
      status: slowStatus,
    });
    render(<MistralLocalPanel />);
    expect(screen.getByText("Ce profil est trop lent sur votre ordinateur.")).toBeInTheDocument();
    expect(actions.install).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Installer le profil Équilibré" }));
    expect(screen.getByRole("alertdialog", { name: "Installer l'IA locale ?" })).toBeInTheDocument();
    expect(actions.install).not.toHaveBeenCalled();
  });

  // Le profil le plus petit n'a aucun profil inférieur à proposer. L'avertissement était
  // pourtant conditionné à l'existence de ce repli : sur une machine lente, l'écran
  // affichait « IA locale prête » et une vitesse de 2,6 tokens/s sans un mot.
  it("avertit d'un profil trop lent même sans profil inférieur à proposer", () => {
    const active = model("light");
    const slowStatus = status(active);
    if (slowStatus.benchmark) {
      slowStatus.benchmark.rating = "too_slow";
      slowStatus.benchmark.tokens_per_second = 2.6;
    }
    setup({ state: "ready", recommendation: recommendation(active), status: slowStatus });
    render(<MistralLocalPanel />);

    expect(screen.getByText(/trop lent sur votre ordinateur/)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Installer le profil/ }),
      "aucun profil inférieur n'existe : rien à proposer",
    ).not.toBeInTheDocument();
  });

  it("n'avertit pas quand les performances sont correctes", () => {
    const active = model("light");
    setup({ state: "ready", recommendation: recommendation(active), status: status(active) });
    render(<MistralLocalPanel />);

    expect(screen.queryByText(/trop lent sur votre ordinateur/)).not.toBeInTheDocument();
  });

  // Le logo doit venir de la propriété `family` renvoyée par le backend, jamais d'une
  // recherche de « Ministral » ou « Qwen » dans le nom affiché.
  it("illustre le modèle avec le logo de sa famille", () => {
    const active = model("light");
    setup({ state: "ready", recommendation: recommendation(active), status: status(active) });
    const { container } = render(<MistralLocalPanel />);

    expect(container.querySelector('img[data-family="mistral"]')).not.toBeNull();
    // Le nom reste écrit : l'information ne dépend jamais du seul logo.
    expect(screen.getByText("Ministral 3 3B")).toBeInTheDocument();
  });

  it("expose la famille du modèle dans la configuration avancée", () => {
    const active = model("balanced");
    setup({ state: "ready", recommendation: recommendation(active), status: status(active) });
    render(<MistralLocalPanel />);

    expect(screen.getByText("Famille")).toBeInTheDocument();
    expect(screen.getByText("Mistral")).toBeInTheDocument();
  });

  it("confirme la suppression avant d'appeler le service", async () => {
    const active = model("light");
    setup({ state: "ready", recommendation: recommendation(active), status: status(active) });
    render(<MistralLocalPanel />);
    await userEvent.click(screen.getByRole("button", { name: "Supprimer le modèle" }));
    await userEvent.click(screen.getByRole("button", { name: "Supprimer" }));
    expect(actions.remove).toHaveBeenCalledWith(active.id);
  });
});
