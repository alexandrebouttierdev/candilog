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
import type { LocalAiViewModel } from "../../../viewmodel/useLocalAiViewModel";

const actions = {
  install: vi.fn(),
  cancel: vi.fn(),
  remove: vi.fn(),
  benchmark: vi.fn(),
  test: vi.fn(),
  reevaluate: vi.fn(),
};

function model(profile: LocalModelProfile): LocalModelDefinition {
  if (profile === "ultra_light") {
    return {
      id: "luth_lfm2_ultra_light",
      profile,
      family: "luth",
      display_name: "Luth LFM2 1.2B",
      repository: "mradermacher/Luth-LFM2-1.2B-GGUF",
      filename: "Luth-LFM2-1.2B.Q4_K_M.gguf",
      local_filename: "luth-lfm2-1.2b-q4_k_m.gguf",
      revision: "a".repeat(40),
      sha256: "b".repeat(64),
      download_size_bytes: 730_895_296,
      estimated_ram_mb: 1_600,
      recommended_ram_mb: 4_096,
      recommended_vram_mb: 2_048,
      recommended_cores: 2,
      context_size: 8192,
      quantization: "Q4_K_M",
      runtime: "llama.cpp",
    };
  }
  const rank = profile === "light" ? "3" : profile === "balanced" ? "8" : "14";
  const cores = profile === "light" ? 4 : profile === "balanced" ? 6 : 8;
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
    recommended_ram_mb: profile === "light" ? 8_192 : profile === "balanced" ? 16_384 : 24_576,
    recommended_vram_mb: profile === "light" ? 4_096 : profile === "balanced" ? 8_192 : 12_288,
    recommended_cores: cores,
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

function setup(overrides: Partial<LocalAiViewModel> = {}): LocalAiViewModel {
  return {
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
  };
}

describe("configuration Mistral Local", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("affiche l'état non configuré sans inventer de recommandation", () => {
    render(<MistralLocalPanel vm={setup()} />);
    expect(document.querySelector("[data-local-ai-state='not_configured']")).toBeInTheDocument();
  });

  it("affiche la détection matérielle", () => {
    render(<MistralLocalPanel vm={setup({ state: "detecting_hardware" })} />);
    expect(screen.getByRole("status", { name: "Détection de la configuration" })).toBeInTheDocument();
  });

  it.each([
    ["light", "Léger"],
    ["balanced", "Équilibré"],
    ["quality", "Qualité"],
  ] as const)("affiche la recommandation %s", (profile, label) => {
    const selected = model(profile);
    render(<MistralLocalPanel vm={setup({ state: "recommendation_ready", recommendation: recommendation(selected) })} />);
    expect(screen.getAllByText(label).length).toBeGreaterThan(0);
    expect(screen.getByRole("button", { name: "Installer l'IA locale" })).toBeInTheDocument();
  });

  it("refuse proprement une machine incompatible", () => {
    render(<MistralLocalPanel vm={setup({ state: "recommendation_ready", recommendation: recommendation(null) })} />);
    expect(screen.getByText("IA locale non recommandée")).toBeInTheDocument();
  });

  it("affiche une progression de téléchargement réelle", () => {
    render(<MistralLocalPanel vm={setup({
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
    })} />);
    expect(screen.getByText("76 %")).toBeInTheDocument();
    expect(screen.getByText(/42 Mo\/s/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Annuler" })).toBeInTheDocument();
  });

  it.each([
    ["verifying", "Vérification du téléchargement"],
    ["installing", "Installation du modèle"],
    ["benchmarking", "Mesure des performances"],
  ] as const)("affiche l'étape %s", (stateValue, title) => {
    render(<MistralLocalPanel vm={setup({ state: stateValue, recommendation: recommendation(model("light")) })} />);
    expect(screen.getByText(title)).toBeInTheDocument();
  });

  it("affiche une erreur de téléchargement", () => {
    render(<MistralLocalPanel vm={setup({ state: "error", error: "Le réseau est indisponible." })} />);
    expect(screen.getByText("Le réseau est indisponible.")).toBeInTheDocument();
  });

  it("affiche le modèle prêt, le benchmark et le résultat de test", () => {
    const active = model("balanced");
    render(
      <MistralLocalPanel
        vm={setup({
          state: "ready",
          recommendation: recommendation(active),
          status: status(active),
          testResult: "Notre équipe d'assistance locale répond.",
        })}
      />,
    );
    expect(screen.getByText("IA locale prête")).toBeInTheDocument();
    expect(screen.getByText("18,4 tokens/s")).toBeInTheDocument();
    // La prose du modèle n'est pas un message d'état : interrogé sur « l'assistance locale »,
    // il répondait « Notre équipe d'assistance locale est entièrement opérationnelle… ».
    // Seul le fait que le test ait abouti est affiché. Le bouton de test vit dans AiHero.
    expect(screen.getByText("Votre modèle local est installé et opérationnel.")).toBeInTheDocument();
    expect(screen.queryByText("Notre équipe d'assistance locale répond.")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Tester l'IA" })).not.toBeInTheDocument();
  });

  it("propose le profil inférieur après un benchmark trop lent sans le télécharger", async () => {
    const active = model("quality");
    const lower = model("balanced");
    const slowStatus = status(active);
    if (slowStatus.benchmark) slowStatus.benchmark.rating = "too_slow";
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      recommendation: { ...recommendation(active), evaluations: [{ model: lower, compatibility: "supported", reason: "Compatible" }] },
      status: slowStatus,
    })} />);
    expect(screen.getByText("Ce profil est trop lent sur votre ordinateur.")).toBeInTheDocument();
    expect(actions.install).not.toHaveBeenCalled();
    // Le bandeau d'avertissement propose le repli ; la liste « Profils disponibles »
    // propose aussi l'installation. Les libellés restent distincts pour éviter l'ambiguïté.
    await userEvent.click(screen.getByRole("button", { name: "Passer au profil Équilibré" }));
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
    render(<MistralLocalPanel vm={setup({ state: "ready", recommendation: recommendation(active), status: slowStatus })} />);

    expect(screen.getByText(/trop lent sur votre ordinateur/)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Installer le profil/ }),
      "aucun profil inférieur n'existe : rien à proposer",
    ).not.toBeInTheDocument();
  });

  it("n'avertit pas quand les performances sont correctes", () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })} />);

    expect(screen.queryByText(/trop lent sur votre ordinateur/)).not.toBeInTheDocument();
  });

  // Le logo doit venir de la propriété `family` renvoyée par le backend, jamais d'une
  // recherche de « Ministral » ou « Luth » dans le nom affiché.
  it("illustre le modèle avec le logo de sa famille", () => {
    const active = model("light");
    const { container } = render(
      <MistralLocalPanel
        vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })}
      />,
    );

    expect(container.querySelector('img[data-family="mistral"]')).not.toBeNull();
    // Le nom reste écrit : l'information ne dépend jamais du seul logo.
    expect(screen.getAllByText("Ministral 3 3B").length).toBeGreaterThan(0);
  });

  it("expose la famille du modèle dans la configuration avancée", () => {
    const active = model("balanced");
    render(
      <MistralLocalPanel
        vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })}
      />,
    );

    expect(screen.getByText("Famille")).toBeInTheDocument();
    expect(screen.getByText("Mistral")).toBeInTheDocument();
  });

  it("annonce le mode ultra léger pour le profil Luth", () => {
    const active = model("ultra_light");
    render(<MistralLocalPanel vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })} />);

    expect(screen.getByText("Ultra léger")).toBeInTheDocument();
    expect(screen.getByText("Mode ultra léger")).toBeInTheDocument();
    expect(screen.getByText(/moins de mémoire/)).toBeInTheDocument();
  });

  it("n'annonce pas le mode ultra léger pour un profil Ministral", () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })} />);

    expect(screen.queryByText("Mode ultra léger")).not.toBeInTheDocument();
  });

  it("affiche la RAM estimée en Go dans la configuration avancée", () => {
    const active = model("ultra_light");
    render(<MistralLocalPanel vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })} />);

    expect(screen.getByText("RAM estimée")).toBeInTheDocument();
    expect(screen.getByText("1,6 Go")).toBeInTheDocument();
    expect(screen.queryByText(/1600 Mo/)).not.toBeInTheDocument();
  });

  // Le backend évalue chaque profil ; l'écran n'en montrait aucun, si bien qu'on ne pouvait
  // ni comparer, ni comprendre pourquoi un profil était écarté.
  it("illustre chaque profil avec une icône distinctive", () => {
    const active = model("ultra_light");
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: status(active),
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "Compatible." },
          { model: model("light"), compatibility: "supported", reason: "Compatible." },
          { model: model("balanced"), compatibility: "supported", reason: "Compatible." },
          { model: model("quality"), compatibility: "unsupported", reason: "Trop lourd." },
        ],
      },
    })} />);

    expect(screen.getAllByText("bolt").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("rocket_launch").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("tune").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("workspace_premium").length).toBeGreaterThanOrEqual(1);
  });

  it("liste chaque profil avec sa compatibilité mesurée", () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: status(active),
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "16 Go de RAM, 8 cœurs." },
          { model: model("balanced"), compatibility: "not_recommended", reason: "Marge insuffisante." },
          { model: model("quality"), compatibility: "unsupported", reason: "24 Go recommandés." },
        ],
      },
    })} />);

    expect(screen.getByText("Recommandé pour votre ordinateur")).toBeInTheDocument();
    expect(screen.getByText("Déconseillé")).toBeInTheDocument();
    expect(screen.getByText("Incompatible")).toBeInTheDocument();
    // La raison vient du backend, jamais d'une règle réécrite dans React.
    // Elle partage la ligne avec le poids disque (« 7 Go sur le disque · 24 Go recommandés. »).
    expect(screen.getAllByText(/sur le disque/).length).toBeGreaterThan(0);
    expect(screen.getByText(/24 Go recommandés/)).toBeInTheDocument();
  });

  it("affiche la config recommandée (RAM, VRAM, cœurs) pour chaque profil", () => {
    const active = model("ultra_light");
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: status(active),
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "Compatible." },
          { model: model("light"), compatibility: "supported", reason: "Compatible." },
        ],
      },
    })} />);

    // Valeurs issues du modèle (registre), pas recalculées dans React.
    expect(screen.getAllByText(/RAM reco 4 Go/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/VRAM reco 2 Go/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/2 cœurs/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/RAM reco 8 Go/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/VRAM reco 4 Go/).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText(/4 cœurs/).length).toBeGreaterThanOrEqual(1);
  });

  it("n'annonce pas recommandé un profil mesuré trop lent", () => {
    const active = model("light");
    const slowStatus = status(active);
    if (slowStatus.benchmark) {
      slowStatus.benchmark.rating = "too_slow";
      slowStatus.benchmark.tokens_per_second = 2.6;
    }
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: slowStatus,
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "16 Go de RAM, 8 cœurs." },
          { model: model("ultra_light"), compatibility: "supported", reason: "Compatible." },
        ],
      },
    })} />);

    // Le matériel suffisait, mais le benchmark a prouvé le contraire : la pastille suit la mesure.
    expect(screen.queryByText("Recommandé pour votre ordinateur")).not.toBeInTheDocument();
    expect(screen.getByText("Déconseillé")).toBeInTheDocument();
    expect(screen.getByText("Mesuré trop lent sur votre ordinateur.")).toBeInTheDocument();
    expect(screen.getByText("Ce profil est trop lent sur votre ordinateur.")).toBeInTheDocument();
  });

  it("interdit d'installer un profil incompatible", () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: status(active),
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "Compatible." },
          { model: model("quality"), compatibility: "unsupported", reason: "Trop lourd." },
        ],
      },
    })} />);

    expect(screen.getByRole("button", { name: /Installer le profil Qualité/ })).toBeDisabled();
    expect(screen.getByText("Profil actif")).toBeInTheDocument();
  });

  it("interdit d'installer un profil déconseillé", () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: status(active),
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "Compatible." },
          { model: model("balanced"), compatibility: "not_recommended", reason: "Marge insuffisante." },
        ],
      },
    })} />);

    expect(screen.getByRole("button", { name: /Installer le profil Équilibré/ })).toBeDisabled();
  });

  it("ne propose pas de repli vers un profil déconseillé après un benchmark trop lent", () => {
    const active = model("balanced");
    const lower = model("light");
    const slowStatus = status(active);
    if (slowStatus.benchmark) slowStatus.benchmark.rating = "too_slow";
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      recommendation: {
        ...recommendation(active),
        evaluations: [{ model: lower, compatibility: "not_recommended", reason: "Marge insuffisante." }],
      },
      status: slowStatus,
    })} />);

    expect(screen.getByText(/trop lent sur votre ordinateur/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Passer au profil/ })).not.toBeInTheDocument();
  });

  it("confirme avant d'installer un autre profil compatible", async () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({
      state: "ready",
      status: status(active),
      recommendation: {
        ...recommendation(active),
        evaluations: [
          { model: active, compatibility: "optimal", reason: "Compatible." },
          { model: model("balanced"), compatibility: "supported", reason: "Compatible." },
        ],
      },
    })} />);

    await userEvent.click(screen.getByRole("button", { name: /Installer le profil Équilibré/ }));
    expect(screen.getByRole("alertdialog", { name: "Installer l'IA locale ?" })).toBeInTheDocument();
    expect(actions.install).not.toHaveBeenCalled();
  });

  it("confirme la suppression avant d'appeler le service", async () => {
    const active = model("light");
    render(<MistralLocalPanel vm={setup({ state: "ready", recommendation: recommendation(active), status: status(active) })} />);
    await userEvent.click(screen.getByRole("button", { name: "Supprimer le modèle" }));
    await userEvent.click(screen.getByRole("button", { name: "Supprimer" }));
    expect(actions.remove).toHaveBeenCalledWith(active.id);
  });
});
