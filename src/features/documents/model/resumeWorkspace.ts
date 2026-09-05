import type {
  ResumeCertificationBlock,
  ResumeDocument,
  ResumeEducationBlock,
  ResumeExperienceBlock,
  ResumeIdentity,
  ResumeLanguageBlock,
  ResumeProfileItem,
  ResumeProjectBlock,
  ResumeSkillGroup,
  ResumeWorkspace,
} from "@/shared/types/generated/documents";

/** Jumeau frontend de `RESUME_WORKSPACE_VERSION` (Rust) : seule cette version est éditable ici. */
export const RESUME_WORKSPACE_VERSION = 1;

/**
 * Cible fermée d'une modification de texte dans le document.
 *
 * Chaque variante désigne un champ précis, jamais un chemin libre : un éditeur qui accepte
 * n'importe quelle clé pourrait écrire hors des champs que `validate_document` (Rust)
 * connaît, ou casser silencieusement le typage d'un bloc.
 */
export type ResumeField =
  | { type: "profile" }
  | { type: "identity"; field: "full_name" | "title" | "headline" | "city" | "phone" | "email" | "website" | "linkedin" | "github" }
  | { type: "experience"; index: number; field: "title" | "company" | "location" | "period" }
  | { type: "experience_bullet"; index: number; item: number }
  | { type: "project"; index: number; field: "name" | "meta" | "url" }
  | { type: "project_bullet"; index: number; item: number }
  | { type: "skill"; group: number; item: number }
  | { type: "education"; index: number; field: "degree" | "school" | "location" | "period" | "description" }
  | { type: "certification"; index: number; field: "name" | "issuer" | "date" }
  | { type: "language"; index: number; field: "name" | "level" };

/** Collection du document qu'une section fermée peut ajouter ou retirer en bloc. */
export type ResumeSectionKind =
  | "experience"
  | "project"
  | "skill_group"
  | "education"
  | "certification"
  | "language";

/**
 * Un workspace CV versionné, reconnu par sa version et la présence de ses collections
 * obligatoires. Un objet `{ schema_version: 1 }` sans document structuré n'en est pas un :
 * seule une composition passée par `prepare_workspace` (Rust) l'est.
 */
function hasResumeDocument(value: unknown): value is { schema_version: number; document: ResumeDocument } {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Partial<ResumeWorkspace>;
  if (candidate.schema_version !== RESUME_WORKSPACE_VERSION) return false;
  if (typeof candidate.document !== "object" || candidate.document === null) return false;
  const document = candidate.document as Partial<ResumeDocument>;
  return (
    Array.isArray(document.experiences) &&
    Array.isArray(document.projects) &&
    Array.isArray(document.skill_groups) &&
    Array.isArray(document.education) &&
    Array.isArray(document.certifications) &&
    Array.isArray(document.languages)
  );
}

/** Met à niveau les workspaces v1 créés avant la bibliothèque éditoriale. */
export function normalizeResumeWorkspace(value: unknown): ResumeWorkspace | null {
  if (!hasResumeDocument(value)) return null;
  const candidate = value as Partial<ResumeWorkspace> & {
    analysis?: Partial<ResumeWorkspace["analysis"]>;
  };
  return {
    ...(candidate as ResumeWorkspace),
    analysis: {
      recap: candidate.analysis?.recap ?? "",
      recommendations: candidate.analysis?.recommendations ?? [],
      content_recommendations: candidate.analysis?.content_recommendations ?? [],
    },
    profile_library: candidate.profile_library ?? [],
    decisions: candidate.decisions ?? { explicitly_added: [], explicitly_removed: [], ignored: [] },
    layout: candidate.layout ?? {
      status: "available",
      used_per_mille: 0,
      remaining_points: 0,
      page_count: 1,
      overflow: false,
    },
    content_recommendations: candidate.content_recommendations ?? [],
    recommendation_error: candidate.recommendation_error ?? null,
  };
}

export function isResumeWorkspace(value: unknown): value is ResumeWorkspace {
  return normalizeResumeWorkspace(value) !== null;
}

function searchKey(value: string): string {
  return value.normalize("NFKD").replace(/[\u0300-\u036f]/g, "").trim().toLocaleLowerCase("fr-FR");
}

export function isProfileItemPresent(document: ResumeDocument, item: ResumeProfileItem): boolean {
  switch (item.content.type) {
    case "skill": {
      const key = searchKey(item.content.name);
      return document.skill_groups.some((group) => group.items.some((skill) => searchKey(skill) === key));
    }
    case "project": {
      const key = searchKey(item.content.value.name);
      return document.projects.some((entry) => entry.id === item.id || searchKey(entry.name) === key);
    }
    case "certification": {
      const key = searchKey(item.content.value.name);
      return document.certifications.some((entry) => entry.id === item.id || searchKey(entry.name) === key);
    }
    case "language": {
      const key = searchKey(item.content.value.name);
      return document.languages.some((entry) => entry.id === item.id || searchKey(entry.name) === key);
    }
  }
}

/** Éléments du profil encore disponibles, indépendamment de leur rang IA. */
export function availableProfileItems(workspace: ResumeWorkspace): ResumeProfileItem[] {
  return workspace.profile_library.filter((item) => {
    if (isProfileItemPresent(workspace.document, item)) return false;
    return !workspace.decisions.explicitly_added.includes(item.id)
      || workspace.decisions.explicitly_removed.includes(item.id);
  });
}

/** Exigences de l'offre absentes à la fois du CV et de la bibliothèque du profil. */
export function missingProfileSkills(workspace: ResumeWorkspace): string[] {
  const profileSkills = workspace.profile_library
    .filter((item) => item.content.type === "skill")
    .map((item) => searchKey(item.content.type === "skill" ? item.content.name : ""));
  return workspace.score.missing.filter((skill) => {
    const requirement = searchKey(skill);
    return !profileSkills.some((candidate) => containsWholeTerm(candidate, requirement));
  });
}

function containsWholeTerm(value: string, term: string): boolean {
  if (!term) return false;
  let offset = value.indexOf(term);
  while (offset >= 0) {
    const before = offset === 0 ? "" : value[offset - 1] ?? "";
    const afterIndex = offset + term.length;
    const after = afterIndex >= value.length ? "" : value[afterIndex] ?? "";
    if (!/[a-z0-9]/i.test(before) && !/[a-z0-9]/i.test(after)) return true;
    offset = value.indexOf(term, offset + 1);
  }
  return false;
}

function addDecision(values: string[], id: string): string[] {
  return values.includes(id) ? values : [...values, id];
}

function removeDecision(values: string[], id: string): string[] {
  return values.filter((value) => value !== id);
}

function insertProfileItem(document: ResumeDocument, item: ResumeProfileItem): ResumeDocument {
  if (isProfileItemPresent(document, item)) return document;
  switch (item.content.type) {
    case "skill": {
      const first = document.skill_groups[0];
      const skill_groups = first
        ? [{ ...first, items: [...first.items, item.content.name] }, ...document.skill_groups.slice(1)]
        : [{ id: "profile-skills", name: "Compétences", items: [item.content.name] }];
      return { ...document, skill_groups };
    }
    case "project": return { ...document, projects: [...document.projects, item.content.value] };
    case "certification": return { ...document, certifications: [...document.certifications, item.content.value] };
    case "language": return { ...document, languages: [...document.languages, item.content.value] };
  }
}

function removeProfileItem(document: ResumeDocument, item: ResumeProfileItem): ResumeDocument {
  switch (item.content.type) {
    case "skill": {
      const key = searchKey(item.content.name);
      return {
        ...document,
        skill_groups: document.skill_groups
          .map((group) => ({ ...group, items: group.items.filter((skill) => searchKey(skill) !== key) }))
          .filter((group) => group.items.length > 0),
      };
    }
    case "project": {
      const key = searchKey(item.content.value.name);
      return { ...document, projects: document.projects.filter((entry) => entry.id !== item.id && searchKey(entry.name) !== key) };
    }
    case "certification": {
      const key = searchKey(item.content.value.name);
      return { ...document, certifications: document.certifications.filter((entry) => entry.id !== item.id && searchKey(entry.name) !== key) };
    }
    case "language": {
      const key = searchKey(item.content.value.name);
      return { ...document, languages: document.languages.filter((entry) => entry.id !== item.id && searchKey(entry.name) !== key) };
    }
  }
}

export function addProfileItem(workspace: ResumeWorkspace, itemId: string): ResumeWorkspace {
  const item = workspace.profile_library.find((candidate) => candidate.id === itemId);
  if (!item || isProfileItemPresent(workspace.document, item)) return workspace;
  return {
    ...workspace,
    document: insertProfileItem(workspace.document, item),
    decisions: {
      ...workspace.decisions,
      explicitly_added: addDecision(workspace.decisions.explicitly_added, item.id),
      explicitly_removed: removeDecision(workspace.decisions.explicitly_removed, item.id),
      ignored: removeDecision(workspace.decisions.ignored, item.id),
    },
    content_recommendations: workspace.content_recommendations.filter((entry) =>
      entry.action.type === "add" ? entry.action.item_id !== item.id : entry.action.add_item_id !== item.id),
  };
}

export function ignoreContentRecommendation(workspace: ResumeWorkspace, recommendationId: string): ResumeWorkspace {
  const recommendation = workspace.content_recommendations.find((entry) => entry.id === recommendationId);
  if (!recommendation) return workspace;
  const itemId = recommendation.action.type === "add" ? recommendation.action.item_id : recommendation.action.add_item_id;
  return {
    ...workspace,
    decisions: { ...workspace.decisions, ignored: addDecision(workspace.decisions.ignored, itemId) },
    content_recommendations: workspace.content_recommendations.filter((entry) => entry.id !== recommendationId),
  };
}

export function applyContentRecommendation(workspace: ResumeWorkspace, recommendationId: string): ResumeWorkspace {
  const recommendation = workspace.content_recommendations.find((entry) => entry.id === recommendationId);
  if (!recommendation) return workspace;
  if (recommendation.action.type === "add") return addProfileItem(workspace, recommendation.action.item_id);
  const action = recommendation.action;
  const added = workspace.profile_library.find((item) => item.id === action.add_item_id);
  const removed = workspace.profile_library.find((item) => item.id === action.remove_item_id);
  if (!added || !removed) return workspace;
  const withoutOld = removeProfileItem(workspace.document, removed);
  return {
    ...workspace,
    document: insertProfileItem(withoutOld, added),
    decisions: {
      ...workspace.decisions,
      explicitly_added: addDecision(workspace.decisions.explicitly_added, added.id),
      explicitly_removed: addDecision(removeDecision(workspace.decisions.explicitly_removed, added.id), removed.id),
      ignored: removeDecision(workspace.decisions.ignored, added.id),
    },
    content_recommendations: workspace.content_recommendations.filter((entry) => entry.id !== recommendationId),
  };
}

/**
 * Remplace un texte vidé par `null` pour un champ optionnel (`Option<String>` côté Rust) —
 * un champ obligatoire reste une chaîne, même vide, pour que `resume_save` la refuse
 * explicitement plutôt que de la faire disparaître silencieusement.
 */
function emptyToNull(value: string): string | null {
  return value.trim() === "" ? null : value;
}

/** Limite les liens persistés dans un CV aux URL HTTP(S) absolues. */
export function safeResumeUrl(value: string | null): string | null {
  if (value === null || value.trim() === "") return null;
  try {
    const parsed = new URL(value);
    if ((parsed.protocol !== "http:" && parsed.protocol !== "https:") || parsed.hostname === "") {
      return null;
    }
    return parsed.toString();
  } catch {
    return null;
  }
}

function updateAt<T>(array: T[], index: number, updater: (item: T) => T | null): T[] | null {
  const item = array[index];
  if (item === undefined) return null;
  const updated = updater(item);
  if (updated === null) return null;
  const next = array.slice();
  next[index] = updated;
  return next;
}

function removeAt<T>(array: T[], index: number): T[] | null {
  if (array[index] === undefined) return null;
  return array.filter((_, position) => position !== index);
}

/** N'affecte que la branche `document` ; un ciblage invalide laisse le workspace intact. */
function withDocument(
  workspace: ResumeWorkspace,
  updater: (document: ResumeDocument) => ResumeDocument | null,
): ResumeWorkspace {
  const document = updater(workspace.document);
  if (document === null) return workspace;
  return { ...workspace, document };
}

function updateIdentity(
  identity: ResumeIdentity,
  field: Extract<ResumeField, { type: "identity" }>["field"],
  value: string,
): ResumeIdentity {
  switch (field) {
    case "full_name": return { ...identity, full_name: value };
    case "title": return { ...identity, title: value };
    case "email": return { ...identity, email: value };
    case "headline": return { ...identity, headline: emptyToNull(value) };
    case "city": return { ...identity, city: emptyToNull(value) };
    case "phone": return { ...identity, phone: emptyToNull(value) };
    case "website": return { ...identity, website: emptyToNull(value) };
    case "linkedin": return { ...identity, linkedin: emptyToNull(value) };
    case "github": return { ...identity, github: emptyToNull(value) };
  }
}

function updateExperience(
  experience: ResumeExperienceBlock,
  field: Extract<ResumeField, { type: "experience" }>["field"],
  value: string,
): ResumeExperienceBlock {
  switch (field) {
    case "title": return { ...experience, title: value };
    case "company": return { ...experience, company: value };
    case "period": return { ...experience, period: value };
    case "location": return { ...experience, location: emptyToNull(value) };
  }
}

function updateProject(
  project: ResumeProjectBlock,
  field: Extract<ResumeField, { type: "project" }>["field"],
  value: string,
): ResumeProjectBlock {
  switch (field) {
    case "name": return { ...project, name: value };
    case "meta": return { ...project, meta: emptyToNull(value) };
    case "url": return { ...project, url: emptyToNull(value) };
  }
}

function updateEducation(
  education: ResumeEducationBlock,
  field: Extract<ResumeField, { type: "education" }>["field"],
  value: string,
): ResumeEducationBlock {
  switch (field) {
    case "degree": return { ...education, degree: value };
    case "school": return { ...education, school: value };
    case "period": return { ...education, period: value };
    case "location": return { ...education, location: emptyToNull(value) };
    case "description": return { ...education, description: emptyToNull(value) };
  }
}

function updateCertification(
  certification: ResumeCertificationBlock,
  field: Extract<ResumeField, { type: "certification" }>["field"],
  value: string,
): ResumeCertificationBlock {
  switch (field) {
    case "name": return { ...certification, name: value };
    case "issuer": return { ...certification, issuer: emptyToNull(value) };
    case "date": return { ...certification, date: emptyToNull(value) };
  }
}

function updateLanguage(
  language: ResumeLanguageBlock,
  field: Extract<ResumeField, { type: "language" }>["field"],
  value: string,
): ResumeLanguageBlock {
  switch (field) {
    case "name": return { ...language, name: value };
    case "level": return { ...language, level: value };
  }
}

/**
 * Applique une modification de texte à une cible fermée du document.
 *
 * Ne clone que la branche touchée (le document, la collection visée, le bloc visé) : le
 * reste du workspace précédent — score, propositions, autres blocs — garde ses références,
 * ce qui rend une comparaison `Object.is` fiable pour l'historique et les recalculs.
 */
export function updateResumeField(
  workspace: ResumeWorkspace,
  field: ResumeField,
  value: string,
): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    switch (field.type) {
      case "profile":
        return { ...document, profile: value };
      case "identity":
        return { ...document, identity: updateIdentity(document.identity, field.field, value) };
      case "experience": {
        const experiences = updateAt(document.experiences, field.index, (experience) =>
          updateExperience(experience, field.field, value));
        return experiences ? { ...document, experiences } : null;
      }
      case "experience_bullet": {
        const experiences = updateAt(document.experiences, field.index, (experience) => {
          const bullets = updateAt(experience.bullets, field.item, () => value);
          return bullets ? { ...experience, bullets } : null;
        });
        return experiences ? { ...document, experiences } : null;
      }
      case "project": {
        const projects = updateAt(document.projects, field.index, (project) =>
          updateProject(project, field.field, value));
        return projects ? { ...document, projects } : null;
      }
      case "project_bullet": {
        const projects = updateAt(document.projects, field.index, (project) => {
          const bullets = updateAt(project.bullets, field.item, () => value);
          return bullets ? { ...project, bullets } : null;
        });
        return projects ? { ...document, projects } : null;
      }
      case "skill": {
        const skill_groups = updateAt(document.skill_groups, field.group, (group) => {
          const items = updateAt(group.items, field.item, () => value);
          return items ? { ...group, items } : null;
        });
        return skill_groups ? { ...document, skill_groups } : null;
      }
      case "education": {
        const education = updateAt(document.education, field.index, (entry) =>
          updateEducation(entry, field.field, value));
        return education ? { ...document, education } : null;
      }
      case "certification": {
        const certifications = updateAt(document.certifications, field.index, (entry) =>
          updateCertification(entry, field.field, value));
        return certifications ? { ...document, certifications } : null;
      }
      case "language": {
        const languages = updateAt(document.languages, field.index, (entry) =>
          updateLanguage(entry, field.field, value));
        return languages ? { ...document, languages } : null;
      }
    }
  });
}

export function addExperienceBullet(workspace: ResumeWorkspace, index: number): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    const experiences = updateAt(document.experiences, index, (experience) => ({
      ...experience,
      bullets: [...experience.bullets, ""],
    }));
    return experiences ? { ...document, experiences } : null;
  });
}

export function removeExperienceBullet(workspace: ResumeWorkspace, index: number, item: number): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    const experiences = updateAt(document.experiences, index, (experience) => {
      const bullets = removeAt(experience.bullets, item);
      return bullets ? { ...experience, bullets } : null;
    });
    return experiences ? { ...document, experiences } : null;
  });
}

export function addProjectBullet(workspace: ResumeWorkspace, index: number): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    const projects = updateAt(document.projects, index, (project) => ({
      ...project,
      bullets: [...project.bullets, ""],
    }));
    return projects ? { ...document, projects } : null;
  });
}

export function removeProjectBullet(workspace: ResumeWorkspace, index: number, item: number): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    const projects = updateAt(document.projects, index, (project) => {
      const bullets = removeAt(project.bullets, item);
      return bullets ? { ...project, bullets } : null;
    });
    return projects ? { ...document, projects } : null;
  });
}

export function addSkill(workspace: ResumeWorkspace, group: number): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    const skill_groups = updateAt(document.skill_groups, group, (skillGroup) => ({
      ...skillGroup,
      items: [...skillGroup.items, ""],
    }));
    return skill_groups ? { ...document, skill_groups } : null;
  });
}

export function removeSkill(workspace: ResumeWorkspace, group: number, item: number): ResumeWorkspace {
  const removed = workspace.document.skill_groups[group]?.items[item];
  const next = withDocument(workspace, (document) => {
    const skill_groups = updateAt(document.skill_groups, group, (skillGroup) => {
      const items = removeAt(skillGroup.items, item);
      return items ? { ...skillGroup, items } : null;
    });
    return skill_groups ? { ...document, skill_groups } : null;
  });
  if (next === workspace || removed === undefined) return next;
  const profileItem = workspace.profile_library.find((candidate) =>
    candidate.content.type === "skill" && searchKey(candidate.content.name) === searchKey(removed));
  if (!profileItem) return next;
  return {
    ...next,
    decisions: {
      ...next.decisions,
      explicitly_added: removeDecision(next.decisions.explicitly_added, profileItem.id),
      explicitly_removed: addDecision(next.decisions.explicitly_removed, profileItem.id),
    },
  };
}

function newBlockId(): string {
  return crypto.randomUUID();
}

function emptyExperience(): ResumeExperienceBlock {
  return { id: newBlockId(), title: "", company: "", location: null, period: "", bullets: [] };
}

function emptyProject(): ResumeProjectBlock {
  return { id: newBlockId(), name: "", meta: null, url: null, bullets: [] };
}

function emptySkillGroup(): ResumeSkillGroup {
  return { id: newBlockId(), name: "", items: [] };
}

function emptyEducation(): ResumeEducationBlock {
  return { id: newBlockId(), degree: "", school: "", location: null, period: "", description: null };
}

function emptyCertification(): ResumeCertificationBlock {
  return { id: newBlockId(), name: "", issuer: null, date: null };
}

function emptyLanguage(): ResumeLanguageBlock {
  return { id: newBlockId(), name: "", level: "" };
}

/**
 * Ajoute un bloc neuf, structurellement complet, à la fin de la collection visée.
 *
 * Le bloc démarre avec des champs obligatoires vides plutôt qu'absents : `resume_save`
 * (Rust) le refusera tel quel, mais jamais faute d'un champ manquant.
 */
export function addSection(workspace: ResumeWorkspace, section: ResumeSectionKind): ResumeWorkspace {
  return withDocument(workspace, (document) => {
    switch (section) {
      case "experience":
        return { ...document, experiences: [...document.experiences, emptyExperience()] };
      case "project":
        return { ...document, projects: [...document.projects, emptyProject()] };
      case "skill_group":
        return { ...document, skill_groups: [...document.skill_groups, emptySkillGroup()] };
      case "education":
        return { ...document, education: [...document.education, emptyEducation()] };
      case "certification":
        return { ...document, certifications: [...document.certifications, emptyCertification()] };
      case "language":
        return { ...document, languages: [...document.languages, emptyLanguage()] };
    }
  });
}

export function removeSection(workspace: ResumeWorkspace, section: ResumeSectionKind, index: number): ResumeWorkspace {
  const removedItems = workspace.profile_library.filter((item) => {
    if (section === "project" && item.content.type === "project") {
      const value = workspace.document.projects[index];
      return value !== undefined && (value.id === item.id || searchKey(value.name) === searchKey(item.content.value.name));
    }
    if (section === "certification" && item.content.type === "certification") {
      const value = workspace.document.certifications[index];
      return value !== undefined && (value.id === item.id || searchKey(value.name) === searchKey(item.content.value.name));
    }
    if (section === "language" && item.content.type === "language") {
      const value = workspace.document.languages[index];
      return value !== undefined && (value.id === item.id || searchKey(value.name) === searchKey(item.content.value.name));
    }
    if (section === "skill_group" && item.content.type === "skill") {
      const skillName = item.content.name;
      return workspace.document.skill_groups[index]?.items.some((skill) => searchKey(skill) === searchKey(skillName)) ?? false;
    }
    return false;
  });
  const next = withDocument(workspace, (document) => {
    switch (section) {
      case "experience": {
        const experiences = removeAt(document.experiences, index);
        return experiences ? { ...document, experiences } : null;
      }
      case "project": {
        const projects = removeAt(document.projects, index);
        return projects ? { ...document, projects } : null;
      }
      case "skill_group": {
        const skill_groups = removeAt(document.skill_groups, index);
        return skill_groups ? { ...document, skill_groups } : null;
      }
      case "education": {
        const education = removeAt(document.education, index);
        return education ? { ...document, education } : null;
      }
      case "certification": {
        const certifications = removeAt(document.certifications, index);
        return certifications ? { ...document, certifications } : null;
      }
      case "language": {
        const languages = removeAt(document.languages, index);
        return languages ? { ...document, languages } : null;
      }
    }
  });
  if (next === workspace || removedItems.length === 0) return next;
  return {
    ...next,
    decisions: removedItems.reduce((decisions, item) => ({
      ...decisions,
      explicitly_added: removeDecision(decisions.explicitly_added, item.id),
      explicitly_removed: addDecision(decisions.explicitly_removed, item.id),
    }), next.decisions),
  };
}

/** Jumeau frontend de `split_bullets` (Rust) : une puce par ligne, sans marqueur ni espace. */
function splitBullets(description: string): string[] {
  return description
    .split(/\r?\n/)
    .map((row) => row.trim().replace(/^[·\-•*\s]+/, "").trim())
    .filter((row) => row.length > 0);
}

/**
 * Annule la décision d'une proposition précise, sans toucher aux autres décisions ni à la
 * pile d'annulation/rétablissement de session.
 *
 * Une proposition déjà `accepted` restaure la partie du document qu'elle avait modifiée
 * (miroir exact de `apply_change`, Rust) : le texte d'origine pour une reformulation, le
 * retrait de la compétence ajoutée pour une compétence manquante. Une proposition `rejected`
 * repasse simplement `pending`, prête à être décidée à nouveau. Le statut ciblé est le seul
 * modifié dans `proposals` ; l'appelant recalcule ensuite score et applicabilité via
 * `documentsService.recalculateResume`, qui préserve le statut de toute autre proposition non
 * `pending` déjà présente.
 */
export function revertProposalDecision(workspace: ResumeWorkspace, proposal_id: string): ResumeWorkspace {
  const proposal = workspace.proposals.find((candidate) => candidate.id === proposal_id);
  if (!proposal || proposal.status === "pending") return workspace;

  let document = workspace.document;
  if (proposal.status === "accepted") {
    if (proposal.target.type === "profile") {
      document = { ...document, profile: proposal.original_text ?? document.profile };
    } else if (proposal.target.type === "experience_description") {
      const experience_id = proposal.target.experience_id;
      const bullets = splitBullets(proposal.original_text ?? "");
      document = {
        ...document,
        experiences: document.experiences.map((experience) =>
          experience.id === experience_id ? { ...experience, bullets } : experience,
        ),
      };
    } else if (proposal.target.type === "skill_group") {
      const group_id = proposal.target.group_id;
      const skill = proposal.proposed_text;
      document = {
        ...document,
        skill_groups: document.skill_groups.map((group) =>
          group.id === group_id ? { ...group, items: group.items.filter((item) => item !== skill) } : group,
        ),
      };
    }
  }

  const proposals = workspace.proposals.map((candidate) =>
    candidate.id === proposal_id ? { ...candidate, status: "pending" as const } : candidate,
  );

  return { ...workspace, document, proposals };
}

/**
 * Workspace de test, autonome et cohérent avec les bornes Rust — réutilisé par les tests
 * du modèle, du ViewModel et (tâches suivantes) des composants d'édition.
 */
export function workspaceFixture(overrides: Partial<ResumeDocument> = {}): ResumeWorkspace {
  const document: ResumeDocument = {
    identity: {
      full_name: "Alex Exemple",
      title: "Développeuse",
      headline: null,
      city: "Paris",
      phone: null,
      email: "alex@exemple.fr",
      website: null,
      linkedin: null,
      github: null,
      extra: [],
    },
    profile: "Profil synthétique.",
    experiences: [
      {
        id: "exp-1",
        title: "Développeuse",
        company: "Candilog",
        location: "Paris",
        period: "Janv. 2024 — Aujourd’hui",
        bullets: ["Impact initial"],
      },
    ],
    projects: [
      {
        id: "proj-1",
        name: "Candilog",
        meta: "TypeScript",
        url: null,
        bullets: ["Fonctionnalité livrée"],
      },
    ],
    skill_groups: [{ id: "group-1", name: "Compétences", items: ["Rust"] }],
    education: [
      {
        id: "edu-1",
        degree: "Master",
        school: "Université",
        location: null,
        period: "2018 — 2020",
        description: null,
      },
    ],
    certifications: [{ id: "cert-1", name: "Certification", issuer: null, date: null }],
    languages: [{ id: "lang-1", name: "Français", level: "Natif" }],
    ...overrides,
  };
  return {
    schema_version: RESUME_WORKSPACE_VERSION,
    document,
    job_offer: { title: "Développeur", skills: [], soft_skills: [], experience: null, keywords: [] },
    analysis: { recap: "", recommendations: [], content_recommendations: [] },
    score: { total: 60, skills: null, experience: null, ats: null, present: [], missing: [] },
    initial_score: 60,
    proposals: [],
    profile_library: [],
    decisions: { explicitly_added: [], explicitly_removed: [], ignored: [] },
    layout: { status: "available", used_per_mille: 0, remaining_points: 0, page_count: 1, overflow: false },
    content_recommendations: [],
    recommendation_error: null,
  };
}
