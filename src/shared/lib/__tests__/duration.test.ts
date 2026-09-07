import { describe, expect, it } from "vitest";
import {
  formatAiSummary,
  formatDuration,
  formatElapsed,
  formatProgressMetrics,
  formatTokens,
  formatTokensPerSecond,
  resolveTokensPerSecond,
} from "../duration";

describe("formatElapsed", () => {
  it("affiche mm:ss sous une heure", () => {
    expect(formatElapsed(12_000)).toBe("00:12");
    expect(formatElapsed(0)).toBe("00:00");
  });

  it("passe en hh:mm:ss au-delà d'une heure", () => {
    expect(formatElapsed(3_661_000)).toBe("01:01:01");
  });
});

describe("formatDuration", () => {
  it("utilise la virgule française pour les dixièmes", () => {
    expect(formatDuration(18_400)).toBe("18,4 s");
  });

  it("arrondit aux secondes entières au-delà de 10 s pile", () => {
    expect(formatDuration(21_000)).toBe("21 s");
  });

  it("passe en minutes au-delà d'une minute", () => {
    expect(formatDuration(70_000)).toBe("1 min 10 s");
    expect(formatDuration(120_000)).toBe("2 min");
  });
});

describe("formatTokens", () => {
  it("sépare les milliers à la française", () => {
    expect(formatTokens(1_024)).toBe("1\u202f024");
    expect(formatTokens(12_480)).toBe("12\u202f480");
  });

  it("laisse un petit nombre sans séparateur", () => {
    expect(formatTokens(640)).toBe("640");
  });
});

describe("formatAiSummary", () => {
  it("accole la durée et les tokens communiqués", () => {
    expect(formatAiSummary("Généré", 18_400, 1_024)).toBe(
      "Généré en 18,4 s · 1\u202f024 tokens",
    );
  });

  it("nomme explicitement une métrique absente", () => {
    expect(formatAiSummary("Analysé", 21_000, null)).toBe(
      "Analysé en 21 s · tokens non communiqués",
    );
  });
});

describe("formatTokensPerSecond", () => {
  it("affiche une décimale à la française", () => {
    expect(formatTokensPerSecond(0.6)).toBe("0,6 tokens/s");
    expect(formatTokensPerSecond(18.4)).toBe("18,4 tokens/s");
  });
});

describe("resolveTokensPerSecond", () => {
  it("préfère le débit natif positif", () => {
    expect(resolveTokensPerSecond(100, 10_000, 2.6)).toBe(2.6);
  });

  it("estime tokens/elapsed quand le débit manque", () => {
    expect(resolveTokensPerSecond(23, 39_000, null)).toBeCloseTo(23 / 39, 5);
  });

  it("reste muet sans tokens ou avant une seconde", () => {
    expect(resolveTokensPerSecond(null, 5_000, null)).toBeNull();
    expect(resolveTokensPerSecond(10, 500, null)).toBeNull();
    expect(resolveTokensPerSecond(0, 5_000, null)).toBeNull();
  });
});

describe("formatProgressMetrics", () => {
  it("accole temps, tokens et débit", () => {
    expect(formatProgressMetrics(39_000, 23, 0.6)).toBe(
      "Temps écoulé : 00:39 · 23 tokens · 0,6 tokens/s",
    );
  });

  it("estime le débit à défaut de mesure native", () => {
    expect(formatProgressMetrics(10_000, 20, null)).toBe(
      "Temps écoulé : 00:10 · 20 tokens · 2,0 tokens/s",
    );
  });

  it("n'affiche que le temps tant que les tokens manquent", () => {
    expect(formatProgressMetrics(12_000, null)).toBe("Temps écoulé : 00:12");
  });
});
