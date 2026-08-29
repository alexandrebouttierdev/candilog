import { z } from "zod";
import type { ImportProfilePreview } from "@/shared/types/generated/profile";

export const importResolutionSchema = z.enum(["keep_existing", "replace", "add_as_new"]);

/** Champ texte facultatif : vide, null ou absent deviennent null. */
const blankToNull = z
  .union([z.string(), z.null(), z.undefined()])
  .transform((value) => {
    if (value == null) return null;
    const trimmed = value.trim();
    return trimmed === "" ? null : trimmed;
  });

/** Lien facultatif : sans protocole on préfixe https, sinon on ignore le lien invalide. */
const importUrl = blankToNull.transform((value) => {
  if (value === null) return null;
  const href = /^(https?):\/\//i.test(value) ? value : `https://${value}`;
  return z.url().safeParse(href).success ? href : null;
});

const importExperienceSchema = z
  .object({
    title: z.string().trim().min(1, "L'intitulé est obligatoire"),
    company: z.string().trim().min(1, "L'entreprise est obligatoire"),
    location: blankToNull,
    start_date: z.string().trim().min(1, "La date de début est obligatoire"),
    end_date: blankToNull,
    current: z.union([z.boolean(), z.undefined()]).transform((value) => value ?? false),
    description: blankToNull,
  })
  .transform((value) => ({
    ...value,
    end_date: value.current ? null : value.end_date,
  }));

const importSkillSchema = z.object({
  name: z.string().trim().min(1, "Le nom est obligatoire"),
});

const importEducationSchema = z.object({
  degree: z.string().trim().min(1, "Le diplôme est obligatoire"),
  school: z.string().trim().min(1, "L'établissement est obligatoire"),
  location: blankToNull,
  start_date: blankToNull,
  end_date: blankToNull,
  description: blankToNull,
});

const importLanguageSchema = z.object({
  name: z.string().trim().min(1, "La langue est obligatoire"),
  level: z
    .union([z.string(), z.null(), z.undefined()])
    .transform((value) => {
      const trimmed = (value ?? "").trim();
      return trimmed === "" ? "Non précisé" : trimmed;
    }),
});

const importProjectSchema = z.object({
  name: z.string().trim().min(1, "Le nom est obligatoire"),
  description: blankToNull,
  url: importUrl,
  technologies: blankToNull,
});

const importCertificationSchema = z.object({
  name: z.string().trim().min(1, "Le nom est obligatoire"),
  issuer: blankToNull,
  date: blankToNull,
  url: importUrl,
});

function willImport(item: { selected: boolean; resolution: string }) {
  return item.selected && item.resolution !== "keep_existing";
}

type ListDecision<T> = {
  id: string;
  selected: boolean;
  value: T;
  existing_index: number | null;
  resolution: z.infer<typeof importResolutionSchema>;
};

const listDecision = <T extends z.ZodType>(value: T) =>
  z
    .object({
      id: z.string().min(1),
      selected: z.boolean(),
      value: z.unknown(),
      existing_index: z.number().int().nonnegative().nullable(),
      resolution: importResolutionSchema,
    })
    .superRefine((item, ctx) => {
      if (!willImport(item)) return;
      const parsed = value.safeParse(item.value);
      if (parsed.success) return;
      for (const issue of parsed.error.issues) {
        ctx.addIssue({
          code: "custom",
          message: issue.message,
          path: ["value", ...issue.path],
        });
      }
    })
    .transform((item) => ({
      ...item,
      value: (willImport(item) ? value.parse(item.value) : item.value) as z.output<T>,
    })) as z.ZodType<ListDecision<z.output<T>>, ListDecision<z.input<T>>>;

export const importProfileRequestSchema = z.object({
  identity: z.array(
    z.object({
      id: z.string().min(1),
      selected: z.boolean(),
      value: z.string(),
      resolution: importResolutionSchema,
    }),
  ),
  experiences: z.array(listDecision(importExperienceSchema)),
  skills: z.array(listDecision(importSkillSchema)),
  education: z.array(listDecision(importEducationSchema)),
  languages: z.array(listDecision(importLanguageSchema)),
  projects: z.array(listDecision(importProjectSchema)),
  certifications: z.array(listDecision(importCertificationSchema)),
});

export type ImportProfileFormValues = z.output<typeof importProfileRequestSchema>;
export type ImportProfileFormInput = z.input<typeof importProfileRequestSchema>;

function hrefOrRaw(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return trimmed;
  const href = /^(https?):\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;
  return z.url().safeParse(href).success ? href : trimmed;
}

function normalizeIdentityValue(id: string, value: string): string {
  return id === "linkedin" || id === "github" || id === "website" ? hrefOrRaw(value) : value;
}

function defaultResolution(has_conflict: boolean, list: boolean) {
  if (has_conflict) return "keep_existing" as const;
  return list ? ("add_as_new" as const) : ("replace" as const);
}

/** Valeurs initiales de la revue : rien n'est appliqué tant que l'utilisateur n'a pas confirmé. */
export function previewToFormValues(preview: ImportProfilePreview): ImportProfileFormInput {
  return {
    identity: preview.identity.map((item) => ({
      id: item.id,
      selected: true,
      value: normalizeIdentityValue(item.id, item.proposed),
      resolution: defaultResolution(item.has_conflict, false),
    })),
    experiences: preview.experiences.map((item) => ({
      id: item.id,
      selected: true,
      value: {
        ...item.proposed,
        end_date: item.proposed.current ? null : item.proposed.end_date,
      },
      existing_index: item.existing_index,
      resolution: defaultResolution(item.has_conflict, true),
    })),
    skills: preview.skills.map((item) => ({
      id: item.id,
      selected: true,
      value: item.proposed,
      existing_index: item.existing_index,
      resolution: defaultResolution(item.has_conflict, true),
    })),
    education: preview.education.map((item) => ({
      id: item.id,
      selected: true,
      value: item.proposed,
      existing_index: item.existing_index,
      resolution: defaultResolution(item.has_conflict, true),
    })),
    languages: preview.languages.map((item) => ({
      id: item.id,
      selected: true,
      value: item.proposed,
      existing_index: item.existing_index,
      resolution: defaultResolution(item.has_conflict, true),
    })),
    projects: preview.projects.map((item) => ({
      id: item.id,
      selected: true,
      value: item.proposed,
      existing_index: item.existing_index,
      resolution: defaultResolution(item.has_conflict, true),
    })),
    certifications: preview.certifications.map((item) => ({
      id: item.id,
      selected: true,
      value: item.proposed,
      existing_index: item.existing_index,
      resolution: defaultResolution(item.has_conflict, true),
    })),
  };
}

type SelectionItem = {
  id?: string;
  selected?: boolean;
  resolution?: string;
  existing_index?: number | null;
};

type SelectionValues = {
  identity?: SelectionItem[];
  experiences?: SelectionItem[];
  skills?: SelectionItem[];
  education?: SelectionItem[];
  languages?: SelectionItem[];
  projects?: SelectionItem[];
  certifications?: SelectionItem[];
};

export const IMPORT_SECTIONS = [
  "identity",
  "experiences",
  "skills",
  "education",
  "languages",
  "projects",
  "certifications",
] as const;

export type ImportSection = (typeof IMPORT_SECTIONS)[number];

export function countChecked(items: SelectionItem[] | undefined): number {
  return (items ?? []).filter((item) => item.selected).length;
}

export function setSectionSelected<T extends { selected: boolean }>(
  items: T[],
  selected: boolean,
): T[] {
  return items.map((item) => ({ ...item, selected }));
}

export function countSelected(values: SelectionValues): number {
  return IMPORT_SECTIONS.reduce(
    (total, section) =>
      total +
      (values[section] ?? []).filter((item) => item.selected && item.resolution !== "keep_existing")
        .length,
    0,
  );
}

/** Cases cochées, y compris celles qui conservent l'existant. */
export function countMarked(values: SelectionValues): number {
  return IMPORT_SECTIONS.reduce((total, section) => total + countChecked(values[section]), 0);
}

const SECTION_ERROR_LABELS: Record<string, string> = {
  identity: "Informations personnelles",
  experiences: "Expériences",
  skills: "Compétences",
  education: "Formations",
  languages: "Langues",
  projects: "Projets",
  certifications: "Certifications",
};

/** Premier motif de refus, pour le bandeau plutôt qu'un message générique. */
export function explainImportErrors(values: unknown): string {
  const parsed = importProfileRequestSchema.safeParse(values);
  if (parsed.success) return "";
  const issue = parsed.error.issues[0];
  if (!issue) {
    return "Certains champs sont incomplets. Corrigez-les dans l'aperçu avant d'importer.";
  }
  const section = String(issue.path[0] ?? "");
  const label = SECTION_ERROR_LABELS[section] ?? "Import";
  return `${label} : ${issue.message}`;
}

export function summarizeImport(
  values: SelectionValues,
  preview?: ImportProfilePreview,
): {
  added: number;
  replaced: number;
  skipped: number;
} {
  let added = 0;
  let replaced = 0;
  let skipped = 0;

  for (const item of values.identity ?? []) {
    if (!item.selected || item.resolution === "keep_existing") {
      skipped += 1;
    } else if (preview?.identity.find((field) => field.id === item.id)?.has_conflict) {
      replaced += 1;
    } else {
      added += 1;
    }
  }

  const lists = [
    values.experiences ?? [],
    values.skills ?? [],
    values.education ?? [],
    values.languages ?? [],
    values.projects ?? [],
    values.certifications ?? [],
  ];
  for (const list of lists) {
    for (const item of list) {
      if (!item.selected || item.resolution === "keep_existing") {
        skipped += 1;
      } else if (item.resolution === "replace" && item.existing_index !== null) {
        replaced += 1;
      } else {
        added += 1;
      }
    }
  }

  return { added, replaced, skipped };
}
