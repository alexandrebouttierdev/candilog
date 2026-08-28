import { describe, expect, it } from "vitest";
import type { Profile } from "@/shared/types/generated/profile";
import { fusionnerProfile } from "../ProfileImportModal";

const vide = (): Profile => ({ identity:{ first_name:"", name:"", email:"", phone:null, city:null, title:null, resume:null, linkedin:null, github:null, website:null }, experiences:[], skills:[], education:[], languages:[], projects:[], certifications:[] });

describe("fusion d'un CV importé", () => {
  it("conserve les coordonnées absentes de l'import et ajoute les entrées", () => {
    const current = vide(); current.identity.email = "camille@example.fr"; current.skills = [{ name:"Rust" }];
    const imported = vide(); imported.identity.first_name = "Camille"; imported.skills = [{ name:"rust" }, { name:"React" }];
    const resultat = fusionnerProfile(current, imported);
    expect(resultat.identity).toMatchObject({ first_name:"Camille", email:"camille@example.fr" });
    expect(resultat.skills.map((v) => v.name)).toEqual(["Rust", "React"]);
  });
});
