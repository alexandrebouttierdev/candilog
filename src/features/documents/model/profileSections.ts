import type { ProfileSection } from "@/shared/types/generated/ai";
import type { Profile } from "@/shared/types/generated/profile";

/** Section proposée à l'interrupteur, avec ce que le profil y contient. */
export interface SectionOption {
  readonly section: ProfileSection;
  readonly label: string;
  readonly count: number;
}

/** « Ce que l'IA peut utiliser » pour un CV (`screens/15`). */
export const RESUME_SECTIONS: ReadonlyArray<readonly [ProfileSection, string]> = [
  ["experiences", "Expériences"],
  ["education", "Formations"],
  ["skills", "Compétences"],
  ["languages", "Langues"],
  ["projects", "Projets"],
  ["certifications", "Certifications"],
  ["interests", "Centres d’intérêt"],
];

/**
 * « Arguments autorisés » pour une lettre (`screens/16`) : les faits du profil dont la
 * lettre peut s'appuyer. Les prétentions salariales n'y figurent pas : le profil ne les
 * connaît pas.
 */
export const LETTER_ARGUMENTS: ReadonlyArray<readonly [ProfileSection, string]> = [
  ["summary", "Présentation"],
  ["experiences", "Expériences"],
  ["skills", "Compétences"],
  ["education", "Formations"],
  ["availability", "Disponibilité"],
  ["projects", "Projets"],
  ["certifications", "Certifications"],
];

export function sectionCount(profile: Profile, section: ProfileSection): number {
  switch (section) {
    case "summary":
      return profile.identity.resume?.trim() ? 1 : 0;
    case "availability":
      return profile.identity.availability?.trim() ? 1 : 0;
    case "experiences":
      return profile.experiences.length;
    case "education":
      return profile.education.length;
    case "skills":
      return profile.skills.length;
    case "languages":
      return profile.languages.length;
    case "projects":
      return profile.projects.length;
    case "certifications":
      return profile.certifications.length;
    case "interests":
      return profile.interests.length;
  }
}

export function sectionOptions(
  profile: Profile | null,
  sections: ReadonlyArray<readonly [ProfileSection, string]>,
): SectionOption[] {
  return sections.map(([section, label]) => ({ section, label, count: profile ? sectionCount(profile, section) : 0 }));
}

/** Ajoute ou retire une section de la liste des exclusions. */
export function toggleSection(excluded: readonly ProfileSection[], section: ProfileSection): ProfileSection[] {
  return excluded.includes(section) ? excluded.filter((value) => value !== section) : [...excluded, section];
}
