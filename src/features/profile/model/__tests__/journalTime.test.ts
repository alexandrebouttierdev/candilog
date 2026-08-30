import { describe, expect, it } from "vitest";
import { formatDuration, formatElapsed, formatJournalTime } from "../formatElapsed";

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

describe("formatJournalTime", () => {
  it("formate un ISO en heure locale", () => {
    expect(formatJournalTime("2026-08-29T14:32:01.000Z")).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });
});
