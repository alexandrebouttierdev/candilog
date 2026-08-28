import { describe, expect, it } from "vitest";
import type { Profil } from "@/shared/types/generated/profil";
import { fusionnerProfil } from "../ProfilImportModal";

const vide = (): Profil => ({ identite:{ prenom:"", nom:"", email:"", telephone:null, ville:null, titre:null, resume:null, linkedin:null, github:null, siteWeb:null }, experiences:[], competences:[], formations:[], langues:[], projets:[], certifications:[] });

describe("fusion d'un CV importé", () => {
  it("conserve les coordonnées absentes de l'import et ajoute les entrées", () => {
    const actuel = vide(); actuel.identite.email = "camille@example.fr"; actuel.competences = [{ nom:"Rust" }];
    const importe = vide(); importe.identite.prenom = "Camille"; importe.competences = [{ nom:"rust" }, { nom:"React" }];
    const resultat = fusionnerProfil(actuel, importe);
    expect(resultat.identite).toMatchObject({ prenom:"Camille", email:"camille@example.fr" });
    expect(resultat.competences.map((v) => v.nom)).toEqual(["Rust", "React"]);
  });
});
