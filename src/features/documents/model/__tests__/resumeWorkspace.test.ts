import { describe, expect, it } from "vitest";
import {
  addExperienceBullet,
  addProjectBullet,
  addSection,
  addSkill,
  isResumeWorkspace,
  removeExperienceBullet,
  removeProjectBullet,
  removeSection,
  removeSkill,
  updateResumeField,
  workspaceFixture,
} from "../resumeWorkspace";

describe("garde de reconnaissance du workspace", () => {
  it("reconnaît uniquement un workspace versionné", () => {
    expect(isResumeWorkspace({ schema_version: 1, document: {} })).toBe(false);
    expect(isResumeWorkspace(workspaceFixture())).toBe(true);
  });

  it("refuse une autre version de schéma", () => {
    const workspace = workspaceFixture();
    expect(isResumeWorkspace({ ...workspace, schema_version: 2 })).toBe(false);
  });
});

describe("mise à jour immuable d'un champ", () => {
  it("modifie une puce sans muter le workspace précédent", () => {
    const before = workspaceFixture();
    const after = updateResumeField(before, { type: "experience_bullet", index: 0, item: 0 }, "Impact doublé");
    expect(after.document.experiences[0]?.bullets[0]).toBe("Impact doublé");
    expect(before.document.experiences[0]?.bullets[0]).toBe("Impact initial");
  });

  it("ne clone que la branche modifiée", () => {
    const before = workspaceFixture();
    const after = updateResumeField(before, { type: "profile" }, "Nouveau profil");
    expect(after.document.profile).toBe("Nouveau profil");
    expect(after.job_offer).toBe(before.job_offer);
    expect(after.score).toBe(before.score);
    expect(after.document.experiences).toBe(before.document.experiences);
    expect(after.document.identity).toBe(before.document.identity);
  });

  it("vide un champ optionnel en null plutôt qu'en chaîne vide", () => {
    const before = workspaceFixture();
    const after = updateResumeField(before, { type: "identity", field: "city" }, "  ");
    expect(after.document.identity.city).toBeNull();
  });

  it("garde une chaîne vide pour un champ obligatoire vidé", () => {
    const before = workspaceFixture();
    const after = updateResumeField(before, { type: "identity", field: "full_name" }, "");
    expect(after.document.identity.full_name).toBe("");
  });

  it("ignore une cible dont l'index n'existe pas", () => {
    const before = workspaceFixture();
    const after = updateResumeField(before, { type: "experience", index: 5, field: "title" }, "Nouveau titre");
    expect(after).toBe(before);
  });
});

describe("opérations fermées sur les listes", () => {
  it("ajoute puis retire une puce d'expérience sans laisser de bloc partiel", () => {
    const base = workspaceFixture();
    const withBullet = addExperienceBullet(base, 0);
    expect(withBullet.document.experiences[0]?.bullets).toEqual(["Impact initial", ""]);

    const withoutBullet = removeExperienceBullet(withBullet, 0, 1);
    expect(withoutBullet.document.experiences[0]?.bullets).toEqual(["Impact initial"]);
  });

  it("ajoute puis retire une puce de projet sans laisser de bloc partiel", () => {
    const base = workspaceFixture();
    const withBullet = addProjectBullet(base, 0);
    expect(withBullet.document.projects[0]?.bullets).toEqual(["Fonctionnalité livrée", ""]);

    const withoutBullet = removeProjectBullet(withBullet, 0, 1);
    expect(withoutBullet.document.projects[0]?.bullets).toEqual(["Fonctionnalité livrée"]);
  });

  it("refuse de retirer une puce de projet à un index absent", () => {
    const base = workspaceFixture();
    const unchanged = removeProjectBullet(base, 0, 9);
    expect(unchanged).toBe(base);
  });

  it("ajoute puis retire une compétence dans un groupe existant", () => {
    const base = workspaceFixture();
    const withSkill = addSkill(base, 0);
    expect(withSkill.document.skill_groups[0]?.items).toEqual(["Rust", ""]);

    const withoutSkill = removeSkill(withSkill, 0, 1);
    expect(withoutSkill.document.skill_groups[0]?.items).toEqual(["Rust"]);
  });

  it("refuse d'ajouter une compétence à un groupe absent", () => {
    const base = workspaceFixture();
    const unchanged = addSkill(base, 9);
    expect(unchanged).toBe(base);
  });

  it("ajoute une nouvelle section structurellement complète", () => {
    const base = workspaceFixture();
    const withCertification = addSection(base, "certification");
    expect(withCertification.document.certifications).toHaveLength(2);
    const created = withCertification.document.certifications[1];
    expect(created).toEqual({ id: created?.id, name: "", issuer: null, date: null });
    expect(created?.id).toBeTruthy();
  });

  it("retire une section existante et refuse un index absent", () => {
    const base = workspaceFixture();
    const withoutLanguage = removeSection(base, "language", 0);
    expect(withoutLanguage.document.languages).toHaveLength(0);

    const unchanged = removeSection(base, "language", 4);
    expect(unchanged).toBe(base);
  });
});
