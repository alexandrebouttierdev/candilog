import { describe, expect, it } from "vitest";
import type { Profile } from "@/shared/types/generated/profile";
import { LETTER_ARGUMENTS, sectionOptions, toggleSection } from "../profileSections";

const identite: Profile["identity"] = {
  first_name: "Jean",
  name: "Rivière",
  email: "jean@exemple.fr",
  phone: null,
  address: null,
  city: null,
  title: null,
  resume: "Chargé d'exploitation",
  birth_date: null,
  age: null,
  availability: "Immédiate",
  desired_contracts: null,
  linkedin: null,
  github: null,
  website: null,
};
const experience = { title: "Technicien", company: "Ker", location: null, start_date: "2020-01", end_date: null, current: true, description: null };
const competence = { name: "Linux", description: null };
const profil: Profile = {
  identity: identite,
  photo: null,
  experiences: [experience, experience],
  skills: [competence, competence, competence],
  education: [{ degree: "BTS", school: "Lycée", location: null, start_date: null, end_date: null, description: null }],
  languages: [],
  projects: [],
  certifications: [],
  interests: [],
};

describe("sections du profil proposées à l'IA", () => {
  it("compte ce que le profil contient pour chaque argument", () => {
    expect(sectionOptions(profil, LETTER_ARGUMENTS).map((option) => option.count)).toEqual([1, 2, 3, 1, 1, 0, 0]);
  });

  it("bascule une section entre autorisée et exclue", () => {
    expect(toggleSection([], "skills")).toEqual(["skills"]);
    expect(toggleSection(["skills", "projects"], "skills")).toEqual(["projects"]);
  });
});
