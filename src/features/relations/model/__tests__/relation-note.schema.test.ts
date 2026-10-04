import { describe, expect, it } from "vitest";
import {
  MAX_NOTE_BODY,
  relationNoteFormSchema,
} from "../schemas/relation-note.schema";

/** Saisie valide minimale, surchargée au cas par cas. */
function saisie(overrides: Record<string, unknown> = {}) {
  return { body: "Rappel du recruteur", noted_on: "02-08-2026", ...overrides };
}

describe("schéma de note de relation", () => {
  it("transforme la date saisie en ISO pour le backend", () => {
    const resultat = relationNoteFormSchema.parse(saisie());
    expect(resultat.noted_on).toBe("2026-08-02");
    expect(resultat.body).toBe("Rappel du recruteur");
  });

  it("supprime les espaces autour du corps", () => {
    expect(relationNoteFormSchema.parse(saisie({ body: "  Entretien  " })).body).toBe(
      "Entretien",
    );
  });

  it("refuse un corps vide ou réduit à des espaces", () => {
    for (const body of ["", "   "]) {
      const resultat = relationNoteFormSchema.safeParse(saisie({ body }));
      expect(resultat.success).toBe(false);
      if (!resultat.success) {
        expect(resultat.error.issues[0]?.message).toBe("La note est obligatoire");
      }
    }
  });

  it("refuse un corps au-delà de la borne du schéma SQLite", () => {
    // La même borne est posée par le service Rust et par le `CHECK` de `relation_notes` :
    // la refuser ici évite un aller-retour IPC pour apprendre la limite.
    const limite = relationNoteFormSchema.safeParse(saisie({ body: "a".repeat(MAX_NOTE_BODY) }));
    expect(limite.success).toBe(true);

    const trop_long = relationNoteFormSchema.safeParse(
      saisie({ body: "a".repeat(MAX_NOTE_BODY + 1) }),
    );
    expect(trop_long.success).toBe(false);
  });

  it("refuse une date illisible ou inexistante", () => {
    for (const noted_on of ["", "32-01-2026", "2026-08-02", "02/08/2026", "pas une date"]) {
      expect(relationNoteFormSchema.safeParse(saisie({ noted_on })).success).toBe(false);
    }
  });
});
