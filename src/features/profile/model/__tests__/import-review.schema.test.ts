import { describe, expect, it } from "vitest";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import {
  countMarked,
  countSelected,
  explainImportErrors,
  importProfileRequestSchema,
  previewToFormValues,
  setSectionSelected,
  summarizeImport,
} from "../import-review.schema";

const preview = (): ImportProfilePreview => ({
  identity: [
    {
      id: "title",
      label: "Titre professionnel",
      proposed: "Lead",
      existing: "Dev",
      has_conflict: true,
    },
  ],
  experiences: [
    {
      id: "exp-0",
      proposed: {
        title: "Dev",
        company: "Lumen",
        location: null,
        start_date: "2022-03",
        end_date: null,
        current: true,
        description: null,
      },
      existing: null,
      existing_index: null,
      has_conflict: false,
    },
  ],
  skills: [],
  education: [],
  languages: [],
  projects: [],
  certifications: [],
  counts: {
    identity: 1,
    experiences: 1,
    skills: 0,
    education: 0,
    languages: 0,
    projects: 0,
    certifications: 0,
  },
});

describe("revue d'import", () => {
  it("prépare un formulaire avec conservation par défaut en cas de conflit", () => {
    const values = previewToFormValues(preview());
    expect(values.identity[0]?.resolution).toBe("keep_existing");
    expect(values.experiences[0]?.resolution).toBe("add_as_new");
    expect(countSelected(values)).toBe(1);
  });

  it("refuse une expérience incomplète sélectionnée", () => {
    const values = previewToFormValues(preview());
    const experience = values.experiences[0];
    if (!experience) throw new Error("expérience attendue");
    experience.value.title = "";
    expect(importProfileRequestSchema.safeParse(values).success).toBe(false);
  });

  it("accepte une expérience ignorée même incomplète", () => {
    const values = previewToFormValues(preview());
    const experience = values.experiences[0];
    if (!experience) throw new Error("expérience attendue");
    experience.value.title = "";
    experience.selected = false;
    expect(importProfileRequestSchema.safeParse(values).success).toBe(true);
  });

  it("retire la date de fin d'un poste actuel à l'ouverture", () => {
    const source = preview();
    const item = source.experiences[0];
    if (!item) throw new Error("expérience attendue");
    item.proposed.end_date = "2026-01";
    item.proposed.current = true;
    const values = previewToFormValues(source);
    expect(values.experiences[0]?.value.end_date).toBeNull();
    expect(importProfileRequestSchema.safeParse(values).success).toBe(true);
  });

  it("compte les cases cochées même si on conserve l'existant", () => {
    const values = previewToFormValues(preview());
    expect(countSelected(values)).toBe(1);
    expect(countMarked(values)).toBe(2);
  });

  it("sélectionne ou ignore toute une section d'un coup", () => {
    const values = previewToFormValues(preview());
    const none = setSectionSelected(values.experiences, false);
    expect(none.every((item) => !item.selected)).toBe(true);
    expect(setSectionSelected(none, true).every((item) => item.selected)).toBe(true);
  });

  it("accepte un CV typique : niveau vide et lien sans protocole", () => {
    const source = preview();
    source.languages = [
      {
        id: "lang-0",
        proposed: { name: "Anglais", level: "" },
        existing: null,
        existing_index: null,
        has_conflict: false,
      },
    ];
    source.projects = [
      {
        id: "proj-0",
        proposed: {
          name: "Candilog",
          description: null,
          url: "github.com/alex/candilog",
          technologies: null,
        },
        existing: null,
        existing_index: null,
        has_conflict: false,
      },
    ];
    const parsed = importProfileRequestSchema.safeParse(previewToFormValues(source));
    expect(parsed.success).toBe(true);
    if (!parsed.success) return;
    expect(parsed.data.languages[0]?.value.level).toBe("Non précisé");
    expect(parsed.data.projects[0]?.value.url).toBe("https://github.com/alex/candilog");
  });

  it("nomme le champ qui bloque l'import", () => {
    const values = previewToFormValues(preview());
    const experience = values.experiences[0];
    if (!experience) throw new Error("expérience attendue");
    experience.value.title = "";
    expect(explainImportErrors(values)).toMatch(/intitulé/i);
  });

  it("compte les remplacements pour la confirmation", () => {
    const values = previewToFormValues(preview());
    const identity = values.identity[0];
    if (!identity) throw new Error("identité attendue");
    identity.resolution = "replace";
    expect(summarizeImport(values, preview())).toEqual({ added: 1, replaced: 1, skipped: 0 });
  });
});
