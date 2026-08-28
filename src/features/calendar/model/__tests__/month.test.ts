import { describe, expect, it } from "vitest";
import {
  gridBounds,
  dateFromIso,
  decalerDays,
  decalerMonth,
  gridDuMonth,
  isoLocal,
  daysDeLaWeek,
  labelDay,
  monthLabel,
  labelWeek,
} from "../month";

describe("grille du mois", () => {
  it("produit toujours 42 cases", () => {
    // Une grille à hauteur variable ferait sauter la mise en page d'un mois à l'autre.
    for (const [year, month] of [
      [2026, 0],
      [2026, 1],
      [2026, 7],
      [2024, 1], // février bissextile
    ] as const) {
      expect(gridDuMonth(year, month)).toHaveLength(42);
    }
  });

  it("commence la semaine le lundi", () => {
    // Le 1er août 2026 est un samedi : la grille doit débuter le lundi 27 juillet.
    const cells = gridDuMonth(2026, 7);
    expect(cells[0]?.iso).toBe("2026-07-27");
  });

  it("commence sur le jour même quand le mois débute un lundi", () => {
    // Le 1er juin 2026 est un lundi : aucune case du mois précédent en tête.
    const cells = gridDuMonth(2026, 5);
    expect(cells[0]?.iso).toBe("2026-06-01");
    expect(cells[0]?.in_month).toBe(true);
  });

  it("marque les jours hors du mois affiché", () => {
    const cells = gridDuMonth(2026, 7);
    expect(cells[0]?.in_month).toBe(false);
    expect(cells.filter((day) => day.in_month)).toHaveLength(31);
  });

  it("marque aujourd'hui, et lui seul", () => {
    const cells = gridDuMonth(2026, 7, new Date(2026, 7, 25));
    const marques = cells.filter((day) => day.today);
    expect(marques).toHaveLength(1);
    expect(marques[0]?.iso).toBe("2026-08-25");
  });

  it("ne marque aucun jour quand la date du jour est hors de la grille", () => {
    expect(gridDuMonth(2026, 7, new Date(2027, 0, 15)).some((day) => day.today)).toBe(
      false,
    );
  });

  it("donne des jours consécutifs, sans trou ni doublon", () => {
    const cells = gridDuMonth(2026, 7);
    const uniques = new Set(cells.map((day) => day.iso));
    expect(uniques.size).toBe(42);

    for (let index = 1; index < cells.length; index += 1) {
      const veille = new Date(`${cells[index - 1]!.iso}T00:00:00Z`);
      const day = new Date(`${cells[index]!.iso}T00:00:00Z`);
      expect(day.getTime() - veille.getTime()).toBe(86_400_000);
    }
  });
});

describe("bornes de la grille", () => {
  it("couvre la grille entière et non le seul mois", () => {
    // Interroger le seul mois laisserait vides les cases de débordement, qui portent
    // pourtant de vrais événements.
    expect(gridBounds(2026, 7)).toEqual({ from: "2026-07-27", to: "2026-09-06" });
  });
});

describe("navigation entre mois", () => {
  it("passe au mois suivant", () => {
    expect(decalerMonth(2026, 7, 1)).toEqual({ year: 2026, month: 8 });
  });

  it("franchit l'année en avant comme en arrière", () => {
    expect(decalerMonth(2026, 11, 1)).toEqual({ year: 2027, month: 0 });
    expect(decalerMonth(2026, 0, -1)).toEqual({ year: 2025, month: 11 });
  });
});

describe("libellé du mois", () => {
  it("nomme le mois en français", () => {
    expect(monthLabel(2026, 7)).toBe("août 2026");
    expect(monthLabel(2026, 0)).toBe("janvier 2026");
  });
});

describe("semaine et jour", () => {
  it("aligne la semaine sur le lundi", () => {
    // Le 28 août 2026 est un vendredi : la semaine commence le lundi 24.
    const days = daysDeLaWeek("2026-08-28", new Date(2026, 7, 28));
    expect(days).toHaveLength(7);
    expect(days[0]?.iso).toBe("2026-08-24");
    expect(days[6]?.iso).toBe("2026-08-30");
    expect(days.filter((day) => day.today)).toHaveLength(1);
  });

  it("décale une clé ISO en heure locale", () => {
    expect(decalerDays("2026-08-31", 1)).toBe("2026-09-01");
    expect(decalerDays("2026-01-01", -1)).toBe("2025-12-31");
  });

  it("round-trip ISO sans glisser en UTC", () => {
    expect(isoLocal(dateFromIso("2026-08-28"))).toBe("2026-08-28");
  });

  it("libellé la semaine et le jour en français", () => {
    expect(labelWeek("2026-08-28")).toMatch(/24.*30.*2026/);
    expect(labelDay("2026-08-28")).toMatch(/vendredi.*28.*août.*2026/i);
  });
});
