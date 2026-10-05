import { describe, expect, it } from "vitest";
import type { Profile } from "@/shared/types/generated/profile";
import { enumerate, profileGaps } from "../profileReadiness";

const identite: Profile["identity"] = {
  first_name: "Jean",
  name: "Rivière",
  email: "jean@exemple.fr",
  phone: null,
  address: null,
  city: null,
  title: null,
  resume: null,
  birth_date: null,
  age: null,
  availability: null,
  desired_contracts: null,
  linkedin: null,
  github: null,
  website: null,
};

const vide: Profile = {
  identity: { ...identite, first_name: "", name: "" },
  photo: null,
  experiences: [],
  skills: [],
  education: [],
  languages: [],
  projects: [],
  certifications: [],
  interests: [],
};

const competence = { name: "Linux", description: null };

describe("ce qui manque au profil pour générer", () => {
  it("ne signale rien dès qu'il y a un nom et un fait", () => {
    expect(profileGaps({ ...vide, identity: identite, skills: [competence] })).toEqual([]);
  });

  it("signale les deux manques d'un profil neuf", () => {
    expect(profileGaps(vide)).toEqual(["votre nom", "au moins une expérience, une formation ou une compétence"]);
  });

  it("réclame le nom complet : un prénom seul ne fait pas d'en-tête de CV", () => {
    expect(profileGaps({ ...vide, identity: { ...identite, name: "  " }, skills: [competence] })).toEqual([
      "votre nom",
    ]);
  });

  it("accepte une formation ou une expérience comme seul fait, pas une langue", () => {
    const formation = { degree: "BTS", school: "Lycée", location: null, start_date: null, end_date: null, description: null };
    expect(profileGaps({ ...vide, identity: identite, education: [formation] })).toEqual([]);
    // Les langues enrichissent un CV sans donner de quoi en écrire un.
    expect(profileGaps({ ...vide, identity: identite, languages: [{ name: "Anglais", level: "C1" }] })).toEqual([
      "au moins une expérience, une formation ou une compétence",
    ]);
  });
});

describe("énumération française", () => {
  it("relie le dernier élément par « et »", () => {
    expect(enumerate([])).toBe("");
    expect(enumerate(["a"])).toBe("a");
    expect(enumerate(["a", "b"])).toBe("a et b");
    expect(enumerate(["a", "b", "c"])).toBe("a, b et c");
  });
});
