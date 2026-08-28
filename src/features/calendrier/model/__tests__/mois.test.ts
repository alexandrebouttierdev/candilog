import { describe, expect, it } from "vitest";
import {
  bornesDeLaGrille,
  dateDepuisIso,
  decalerJours,
  decalerMois,
  grilleDuMois,
  isoLocal,
  joursDeLaSemaine,
  libelleJour,
  libelleMois,
  libelleSemaine,
} from "../mois";

describe("grille du mois", () => {
  it("produit toujours 42 cases", () => {
    // Une grille à hauteur variable ferait sauter la mise en page d'un mois à l'autre.
    for (const [annee, mois] of [
      [2026, 0],
      [2026, 1],
      [2026, 7],
      [2024, 1], // février bissextile
    ] as const) {
      expect(grilleDuMois(annee, mois)).toHaveLength(42);
    }
  });

  it("commence la semaine le lundi", () => {
    // Le 1er août 2026 est un samedi : la grille doit débuter le lundi 27 juillet.
    const cases = grilleDuMois(2026, 7);
    expect(cases[0]?.iso).toBe("2026-07-27");
  });

  it("commence sur le jour même quand le mois débute un lundi", () => {
    // Le 1er juin 2026 est un lundi : aucune case du mois précédent en tête.
    const cases = grilleDuMois(2026, 5);
    expect(cases[0]?.iso).toBe("2026-06-01");
    expect(cases[0]?.dansLeMois).toBe(true);
  });

  it("marque les jours hors du mois affiché", () => {
    const cases = grilleDuMois(2026, 7);
    expect(cases[0]?.dansLeMois).toBe(false);
    expect(cases.filter((jour) => jour.dansLeMois)).toHaveLength(31);
  });

  it("marque aujourd'hui, et lui seul", () => {
    const cases = grilleDuMois(2026, 7, new Date(2026, 7, 25));
    const marques = cases.filter((jour) => jour.aujourdhui);
    expect(marques).toHaveLength(1);
    expect(marques[0]?.iso).toBe("2026-08-25");
  });

  it("ne marque aucun jour quand la date du jour est hors de la grille", () => {
    expect(grilleDuMois(2026, 7, new Date(2027, 0, 15)).some((jour) => jour.aujourdhui)).toBe(
      false,
    );
  });

  it("donne des jours consécutifs, sans trou ni doublon", () => {
    const cases = grilleDuMois(2026, 7);
    const uniques = new Set(cases.map((jour) => jour.iso));
    expect(uniques.size).toBe(42);

    for (let index = 1; index < cases.length; index += 1) {
      const veille = new Date(`${cases[index - 1]!.iso}T00:00:00Z`);
      const jour = new Date(`${cases[index]!.iso}T00:00:00Z`);
      expect(jour.getTime() - veille.getTime()).toBe(86_400_000);
    }
  });
});

describe("bornes de la grille", () => {
  it("couvre la grille entière et non le seul mois", () => {
    // Interroger le seul mois laisserait vides les cases de débordement, qui portent
    // pourtant de vrais événements.
    expect(bornesDeLaGrille(2026, 7)).toEqual({ from: "2026-07-27", to: "2026-09-06" });
  });
});

describe("navigation entre mois", () => {
  it("passe au mois suivant", () => {
    expect(decalerMois(2026, 7, 1)).toEqual({ annee: 2026, mois: 8 });
  });

  it("franchit l'année en avant comme en arrière", () => {
    expect(decalerMois(2026, 11, 1)).toEqual({ annee: 2027, mois: 0 });
    expect(decalerMois(2026, 0, -1)).toEqual({ annee: 2025, mois: 11 });
  });
});

describe("libellé du mois", () => {
  it("nomme le mois en français", () => {
    expect(libelleMois(2026, 7)).toBe("août 2026");
    expect(libelleMois(2026, 0)).toBe("janvier 2026");
  });
});

describe("semaine et jour", () => {
  it("aligne la semaine sur le lundi", () => {
    // Le 28 août 2026 est un vendredi : la semaine commence le lundi 24.
    const jours = joursDeLaSemaine("2026-08-28", new Date(2026, 7, 28));
    expect(jours).toHaveLength(7);
    expect(jours[0]?.iso).toBe("2026-08-24");
    expect(jours[6]?.iso).toBe("2026-08-30");
    expect(jours.filter((jour) => jour.aujourdhui)).toHaveLength(1);
  });

  it("décale une clé ISO en heure locale", () => {
    expect(decalerJours("2026-08-31", 1)).toBe("2026-09-01");
    expect(decalerJours("2026-01-01", -1)).toBe("2025-12-31");
  });

  it("round-trip ISO sans glisser en UTC", () => {
    expect(isoLocal(dateDepuisIso("2026-08-28"))).toBe("2026-08-28");
  });

  it("libellé la semaine et le jour en français", () => {
    expect(libelleSemaine("2026-08-28")).toMatch(/24.*30.*2026/);
    expect(libelleJour("2026-08-28")).toMatch(/vendredi.*28.*août.*2026/i);
  });
});
