import { describe, expect, it } from "vitest";
import { depuisEntretien, depuisRelance, grouperParJour } from "../evenement";
import type { Entretien } from "@/features/entretiens/services/entretien.service";
import type { Relance } from "@/features/relances/services/relance.service";

function entretien(id: string, horodatage: string): Entretien {
  return {
    id,
    candidatureId: "c1",
    candidaturePoste: "Développeur Frontend",
    entrepriseNom: "Nova Digital",
    contactId: null,
    contactNom: null,
    dateEntretien: horodatage,
    type: "Visio",
    lieu: null,
    notes: null,
    compteRendu: null,
    analyseIa: null,
    createdAt: "2026-08-20T00:00:00Z",
    updatedAt: "2026-08-20T00:00:00Z",
  };
}

function relance(id: string, date: string): Relance {
  return {
    id,
    candidatureId: "c2",
    candidaturePoste: "Product Designer",
    entrepriseNom: "Atlas Studio",
    dateRelance: date,
    type: "Email",
    notes: null,
    createdAt: "2026-08-20T00:00:00Z",
  };
}

describe("conversion en événement", () => {
  it("donne à l'entretien la tonalité de l'avancement et son heure", () => {
    const evenement = depuisEntretien(entretien("e1", "2026-08-25T14:00:00+02:00"));

    expect(evenement).toMatchObject({
      genre: "entretien",
      jour: "2026-08-25",
      heure: "14:00",
      tone: "success",
      libelle: "Développeur Frontend",
      detail: "Nova Digital",
    });
  });

  it("donne à la relance la tonalité de ce qui est à traiter, sans heure", () => {
    // Une relance se programme au jour : afficher « 00:00 » suggérerait un créneau.
    const evenement = depuisRelance(relance("r1", "2026-08-27"));

    expect(evenement).toMatchObject({ genre: "relance", jour: "2026-08-27", tone: "warning" });
    expect(evenement.heure).toBeNull();
  });

  it("retombe sur un libellé générique quand la candidature n'est pas résolue", () => {
    const orphelin = { ...entretien("e1", "2026-08-25T14:00:00+02:00"), candidaturePoste: null };
    expect(depuisEntretien(orphelin).libelle).toBe("Entretien");
  });
});

describe("regroupement par jour", () => {
  it("range chaque événement dans sa journée", () => {
    const parJour = grouperParJour([
      depuisEntretien(entretien("e1", "2026-08-25T14:00:00+02:00")),
      depuisRelance(relance("r1", "2026-08-27")),
      depuisRelance(relance("r2", "2026-08-25")),
    ]);

    expect([...parJour.keys()].sort()).toEqual(["2026-08-25", "2026-08-27"]);
    expect(parJour.get("2026-08-25")).toHaveLength(2);
  });

  it("place les relances avant les entretiens d'une même journée", () => {
    // Une relance se traite quand on veut ; un entretien a un créneau, qui vient après.
    const parJour = grouperParJour([
      depuisEntretien(entretien("e1", "2026-08-25T09:00:00+02:00")),
      depuisRelance(relance("r1", "2026-08-25")),
    ]);

    expect(parJour.get("2026-08-25")?.map((e) => e.genre)).toEqual(["relance", "entretien"]);
  });

  it("trie les entretiens d'une journée par heure croissante", () => {
    const parJour = grouperParJour([
      depuisEntretien(entretien("e2", "2026-08-25T16:00:00+02:00")),
      depuisEntretien(entretien("e1", "2026-08-25T09:00:00+02:00")),
    ]);

    expect(parJour.get("2026-08-25")?.map((e) => e.heure)).toEqual(["09:00", "16:00"]);
  });

  it("ne crée aucune journée pour une liste vide", () => {
    expect(grouperParJour([]).size).toBe(0);
  });
});
