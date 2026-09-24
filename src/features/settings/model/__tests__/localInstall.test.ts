import { describe, expect, it } from "vitest";
import type { ManagedModelDefinition, ManagedOllamaDownloadProgress } from "@/shared/types/generated/ai";
import { currentStep, formatBytes, formatRemaining, laterStep, modelMeters, stepState, throughput } from "../localInstall";

const definition: ManagedModelDefinition = {
  id: "ministral3_light",
  category: "light",
  publisher: "mistral",
  publisher_label: "Mistral AI",
  display_name: "Ministral 3 · 3B",
  description: "",
  ollama_tag: "ministral-3:3b",
  approximate_download_bytes: 2_100_000_000,
  recommended_ram_gb: 4,
};

function event(kind: ManagedOllamaDownloadProgress["kind"]): ManagedOllamaDownloadProgress {
  return { kind, model_id: null, state: "downloading", downloaded_bytes: 0, total_bytes: 0, progress: 0, label: "" };
}

describe("installation de l'IA locale", () => {
  it("écrit les tailles en Mo puis en Go", () => {
    expect(formatBytes(42_000_000)).toBe("42 Mo");
    expect(formatBytes(2_100_000_000)).toBe("2,1 Go");
  });

  it("dessine la catégorie du catalogue et la mémoire recommandée", () => {
    expect(modelMeters(definition)).toEqual({ speed: 3, quality: 2, memory: 1 });
    expect(modelMeters({ ...definition, category: "max_quality", recommended_ram_gb: 16 })).toEqual({
      speed: 1,
      quality: 3,
      memory: 3,
    });
  });

  it("passe du moteur au modèle au premier événement de modèle", () => {
    expect(currentStep(null)).toBe("engine");
    expect(currentStep(event("runtime"))).toBe("engine");
    expect(currentStep(event("model"))).toBe("model");
  });

  it("ne revient jamais à une étape passée", () => {
    expect(laterStep("model", "engine")).toBe("model");
    expect(laterStep("model", "verify")).toBe("verify");
    expect(laterStep(null, "engine")).toBe("engine");
  });

  it("marque faites les étapes qui précèdent l'étape en cours", () => {
    expect(stepState("engine", "model")).toBe("done");
    expect(stepState("model", "model")).toBe("running");
    expect(stepState("verify", "model")).toBe("pending");
    expect(stepState("engine", null)).toBe("pending");
  });

  it("calcule le débit et le temps restant sur les octets reçus", () => {
    expect(throughput(0, 100, 5_000)).toBeNull();
    expect(throughput(50_000_000, 150_000_000, 5_000)).toEqual({ bytesPerSecond: 10_000_000, remainingSeconds: 10 });
    expect(formatRemaining(185)).toBe("3 min 05");
    expect(formatRemaining(12)).toBe("12 s");
  });
});
