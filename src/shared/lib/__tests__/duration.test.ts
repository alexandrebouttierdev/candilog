import { describe, expect, it } from "vitest";
import {
  formatAiSummary,
  formatDuration,
  formatElapsed,
  formatTokens,
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
