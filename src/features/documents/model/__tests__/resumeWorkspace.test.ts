import { describe, expect, it } from "vitest";
import {
  addExperienceBullet,
  addProfileItem,
  addProjectBullet,
  addSection,
  addSkill,
  applyContentRecommendation,
  availableProfileItems,
  ignoreContentRecommendation,
  isResumeWorkspace,
  missingProfileSkills,
  normalizeResumeWorkspace,
  removeExperienceBullet,
  removeProjectBullet,
  removeSection,
  removeSkill,
  safeResumeUrl,
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

  it("met à niveau une version enregistrée avant la bibliothèque éditoriale", () => {
    const current = workspaceFixture();
    const legacy = { ...current } as Record<string, unknown>;
    delete legacy.profile_library;
    delete legacy.decisions;
    delete legacy.layout;
    delete legacy.content_recommendations;
    legacy.analysis = { recap: "Historique", recommendations: [] };
    const normalized = normalizeResumeWorkspace(legacy);
    expect(normalized?.document).toEqual(current.document);
    expect(normalized?.profile_library).toEqual([]);
    expect(normalized?.analysis.content_recommendations).toEqual([]);
  });
});

describe("validation des URL affichées", () => {
  it("accepte uniquement les URL HTTP et HTTPS absolues", () => {
    expect(safeResumeUrl("https://example.test/path")).toBe("https://example.test/path");
    expect(safeResumeUrl("http://example.test")).toBe("http://example.test/");
    expect(safeResumeUrl("javascript:alert(1)")).toBeNull();
    expect(safeResumeUrl("data:text/html,attaque")).toBeNull();
    expect(safeResumeUrl("/chemin-relatif")).toBeNull();
    expect(safeResumeUrl("pas une url")).toBeNull();
    expect(safeResumeUrl(null)).toBeNull();
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

describe("bibliothèque éditoriale du profil", () => {
  function editorialWorkspace() {
    const base = workspaceFixture({ projects: [], skill_groups: [], certifications: [], languages: [] });
    return {
      ...base,
      score: { ...base.score, missing: ["Docker", "Kubernetes"] },
      profile_library: [
        { id: "skill-docker", label: "Docker", detail: null, content: { type: "skill" as const, name: "Docker" } },
        { id: "project-lab", label: "Homelab", detail: null, content: { type: "project" as const, value: { id: "project-lab", name: "Homelab", meta: null, url: null, bullets: [] } } },
      ],
      content_recommendations: [
        { id: "recommend-docker", label: "Docker", reason: "Demandé", relevance: "very_relevant" as const, action: { type: "add" as const, item_id: "skill-docker" }, layout_after: base.layout },
      ],
    };
  }

  it("affiche un élément absent, puis le retire des suggestions dès son ajout", () => {
    const before = editorialWorkspace();
    expect(availableProfileItems(before).map((item) => item.id)).toContain("skill-docker");
    const after = addProfileItem(before, "skill-docker");
    expect(after.document.skill_groups[0]?.items).toEqual(["Docker"]);
    expect(availableProfileItems(after).map((item) => item.id)).not.toContain("skill-docker");
    expect(after.decisions.explicitly_added).toContain("skill-docker");
  });

  it("rend une compétence disponible après son retrait et préserve cette intention", () => {
    const added = addProfileItem(editorialWorkspace(), "skill-docker");
    const removed = removeSkill(added, 0, 0);
    expect(availableProfileItems(removed).map((item) => item.id)).toContain("skill-docker");
    expect(removed.decisions.explicitly_removed).toContain("skill-docker");
    expect(removed.decisions.explicitly_added).not.toContain("skill-docker");
    const readded = addProfileItem(removed, "skill-docker");
    expect(readded.document.skill_groups[0]?.items).toEqual(["Docker"]);
    expect(readded.decisions.explicitly_removed).not.toContain("skill-docker");
  });

  it("ne repropose pas un élément explicitement ajouté puis reformulé", () => {
    const added = addProfileItem(editorialWorkspace(), "skill-docker");
    const edited = updateResumeField(added, { type: "skill", group: 0, item: 0 }, "Docker avancé");
    expect(availableProfileItems(edited).map((item) => item.id)).not.toContain("skill-docker");
  });

  it("ignore une recommandation sans masquer l'élément des suggestions normales", () => {
    const ignored = ignoreContentRecommendation(editorialWorkspace(), "recommend-docker");
    expect(ignored.content_recommendations).toEqual([]);
    expect(ignored.decisions.ignored).toEqual(["skill-docker"]);
    expect(availableProfileItems(ignored).map((item) => item.id)).toContain("skill-docker");
  });

  it("accepte une recommandation uniquement à la demande de l'utilisateur", () => {
    const before = editorialWorkspace();
    expect(before.document.skill_groups).toEqual([]);
    const after = applyContentRecommendation(before, "recommend-docker");
    expect(after.document.skill_groups[0]?.items).toEqual(["Docker"]);
  });

  it("signale une exigence absente du profil sans proposer de l'ajouter au CV", () => {
    const workspace = editorialWorkspace();
    expect(missingProfileSkills(workspace)).toEqual(["Kubernetes"]);
    expect(workspace.content_recommendations.every((item) => item.label !== "Kubernetes")).toBe(true);
  });
});
