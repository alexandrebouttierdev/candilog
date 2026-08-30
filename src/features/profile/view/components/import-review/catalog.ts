import type { ImportProfilePreview } from "@/shared/types/generated/profile";
import type { ImportSection } from "../../../model/import-review.schema";

export const SECTION_LABELS: Record<ImportSection, string> = {
  identity: "Informations personnelles",
  experiences: "Expériences",
  skills: "Compétences",
  education: "Formations",
  languages: "Langues",
  projects: "Projets",
  certifications: "Certifications",
};

export const SECTION_ARIA: Record<ImportSection, string> = {
  identity: "les informations personnelles",
  experiences: "les expériences",
  skills: "les compétences",
  education: "les formations",
  languages: "les langues",
  projects: "les projets",
  certifications: "les certifications",
};

export const SECTION_ICONS: Record<ImportSection, string> = {
  identity: "badge",
  experiences: "work_history",
  skills: "psychology",
  education: "school",
  languages: "translate",
  projects: "rocket_launch",
  certifications: "workspace_premium",
};

export type CatalogRow = {
  section: ImportSection;
  index: number;
  id: string;
  title: string;
  subtitle?: string;
  conflict: boolean;
};

export function catalogOf(preview: ImportProfilePreview): CatalogRow[] {
  return [
    ...preview.identity.map((item, index) => ({
      section: "identity" as const,
      index,
      id: item.id,
      title: item.label,
      conflict: item.has_conflict,
    })),
    ...preview.experiences.map((item, index) => ({
      section: "experiences" as const,
      index,
      id: item.id,
      title: item.proposed.title,
      subtitle: `${item.proposed.company}${item.proposed.start_date ? ` · ${item.proposed.start_date}` : ""}`,
      conflict: item.has_conflict,
    })),
    ...preview.skills.map((item, index) => ({
      section: "skills" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      conflict: item.has_conflict,
    })),
    ...preview.education.map((item, index) => ({
      section: "education" as const,
      index,
      id: item.id,
      title: item.proposed.degree,
      subtitle: item.proposed.school,
      conflict: item.has_conflict,
    })),
    ...preview.languages.map((item, index) => ({
      section: "languages" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      conflict: item.has_conflict,
    })),
    ...preview.projects.map((item, index) => ({
      section: "projects" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      ...(item.proposed.technologies ? { subtitle: item.proposed.technologies } : {}),
      conflict: item.has_conflict,
    })),
    ...preview.certifications.map((item, index) => ({
      section: "certifications" as const,
      index,
      id: item.id,
      title: item.proposed.name,
      ...(item.proposed.issuer ? { subtitle: item.proposed.issuer } : {}),
      conflict: item.has_conflict,
    })),
  ];
}

export function blockId(id: string) {
  return `import-block-${id}`;
}
