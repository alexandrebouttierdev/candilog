import { describe, expect, it } from "vitest";
import {
  dateDepuisHorodatage,
  heureDepuisHorodatage,
  heureValide,
  jourDe,
  versDateAffichee,
  versDateIso,
  versHorodatage,
} from "../dates";

describe("conversion de date", () => {
  it("convertit le format saisi en format base", () => {
    expect(versDateIso("20-08-2026")).toBe("2026-08-20");
  });

  it("fait l'aller-retour sans perte", () => {
    expect(versDateAffichee(versDateIso("29-02-2024") as string)).toBe("29-02-2024");
  });

  it("refuse une date qui n'existe pas", () => {
    // `new Date("2026-02-31")` décale au 3 mars sans erreur : seule la comparaison de la
    // valeur relue permet de refuser un jour inexistant.
    expect(versDateIso("31-02-2026")).toBeNull();
    expect(versDateIso("29-02-2026")).toBeNull();
  });

  it("accepte le 29 février d'une année bissextile", () => {
    expect(versDateIso("29-02-2024")).toBe("2024-02-29");
  });

  it("refuse une saisie hors format", () => {
    for (const saisie of ["2026-08-20", "20/08/2026", "20 août 2026", "20-8-2026", ""]) {
      expect(versDateIso(saisie)).toBeNull();
    }
  });

  it("laisse intacte une date déjà affichable qu'on ne sait pas relire", () => {
    // Les lignes héritées portent parfois un horodatage ISO complet : mieux vaut l'afficher
    // tel quel qu'afficher « Invalid Date ».
    expect(versDateAffichee("2026-08-20T10:00:00Z")).toBe("20-08-2026");
    expect(versDateAffichee("date inconnue")).toBe("date inconnue");
  });
});


describe("heure", () => {
  it("accepte une heure sur 24 heures", () => {
    for (const heure of ["00:00", "09:30", "14:00", "23:59"]) {
      expect(heureValide(heure)).toBe(true);
    }
  });

  it("refuse une heure impossible ou mal formée", () => {
    for (const heure of ["24:00", "12:60", "9:30", "14h00", ""]) {
      expect(heureValide(heure)).toBe(false);
    }
  });
});

describe("horodatage d'un entretien", () => {
  it("compose la date, l'heure et le décalage local", () => {
    // Le décalage est indispensable : sans lui, un entretien saisi à 14 h s'afficherait à
    // 12 h ou 16 h selon le fuseau où la base est relue.
    const horodatage = versHorodatage("25-08-2026", "14:00");

    expect(horodatage).toMatch(/^2026-08-25T14:00:00[+-]\d{2}:\d{2}$/);
  });

  it("refuse une date ou une heure invalide", () => {
    expect(versHorodatage("31-02-2026", "14:00")).toBeNull();
    expect(versHorodatage("25-08-2026", "25:00")).toBeNull();
  });

  it("fait l'aller-retour vers les champs du formulaire", () => {
    const horodatage = versHorodatage("25-08-2026", "09:05") as string;

    expect(dateDepuisHorodatage(horodatage)).toBe("25-08-2026");
    expect(heureDepuisHorodatage(horodatage)).toBe("09:05");
  });

  it("extrait le jour d'un horodatage comme d'une date nue", () => {
    // Le calendrier regroupe entretiens et relances par journée, alors que les uns portent
    // une heure et les autres non.
    expect(jourDe("2026-08-25T14:00:00+02:00")).toBe("2026-08-25");
    expect(jourDe("2026-08-25")).toBe("2026-08-25");
  });
});
