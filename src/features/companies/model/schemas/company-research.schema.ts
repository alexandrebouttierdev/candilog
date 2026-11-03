import { z } from "zod";
import type {
  CompanyCandidate,
  CompanyResearchResult,
  CompanySearchResult,
  ResearchedField,
} from "@/shared/types/generated/ai";
import type { CompanySize } from "@/shared/types/generated/companies";
import type { CompanyFormInput } from "./company-form.schema";

/**
 * Données de « Compléter avec l'IA ».
 *
 * La réponse du modèle est déjà validée et recadrée en Rust (`docs/AI.md`) : ces schémas
 * vérifient la charge utile reçue par l'IPC avant de l'afficher — forme, bornes, URL
 * HTTP(S) seulement pour les liens qu'un clic ouvrira — et décrivent le formulaire de
 * vérification. Les types générés par ts-rs restent la référence : chaque schéma y est
 * confronté à la compilation.
 */

const httpUrl = z
  .string()
  .max(2048)
  .refine((value) => {
    try {
      return ["http:", "https:"].includes(new URL(value).protocol);
    } catch {
      return false;
    }
  }, "Lien invalide");

const shortText = z.string().max(500);

const researchedFieldSchema = z.object({
  value: shortText.nullable(),
  origin: z.enum(["registry", "wikidata", "web_search", "ai"]).nullable(),
  source_url: httpUrl.nullable(),
  note: shortText.nullable(),
}) satisfies z.ZodType<ResearchedField>;

export const companyCandidateSchema = z.object({
  siret: z.string().regex(/^\d{14}$/),
  siren: z.string().regex(/^\d{9}$/),
  name: z.string().min(1).max(300),
  trade_name: z.string().max(300).nullable(),
  city: z.string().max(200).nullable(),
  postal_code: z.string().max(20).nullable(),
  address: shortText.nullable(),
  head_office: z.boolean(),
  activity_code: z.string().max(20).nullable(),
  activity_label: z.string().max(200).nullable(),
  legal_category: z.string().max(10).nullable(),
  size_category: z.string().max(10).nullable(),
  workforce_range: z.string().max(10).nullable(),
  open_establishments: z.number().int().nonnegative().nullable(),
  source_url: httpUrl,
}) satisfies z.ZodType<CompanyCandidate>;

export const companySearchResultSchema = z.object({
  candidates: z.array(companyCandidateSchema).max(25),
  confident: z.boolean(),
  truncated: z.boolean(),
}) satisfies z.ZodType<CompanySearchResult>;

export const companyResearchResultSchema = z.object({
  fields: z.object({
    name: researchedFieldSchema,
    sector_id: researchedFieldSchema,
    company_type_id: researchedFieldSchema,
    company_size: researchedFieldSchema,
    website: researchedFieldSchema,
    city: researchedFieldSchema,
    address: researchedFieldSchema,
  }),
  sources: z.array(z.object({ label: z.string().max(300), url: httpUrl })).max(20),
  warnings: z.array(shortText).max(10),
  ai_used: z.boolean(),
}) satisfies z.ZodType<CompanyResearchResult>;

/** Champs du formulaire entreprise que la recherche peut remplir. */
export const RESEARCHED_FIELDS = [
  "name",
  "sector_id",
  "company_type_id",
  "company_size",
  "website",
  "city",
  "address",
] as const;

export type ResearchedFieldName = (typeof RESEARCHED_FIELDS)[number];

const COMPANY_SIZES = ["MICRO", "TPE", "PME", "ETI", "LARGE", "UNKNOWN"] as const satisfies readonly CompanySize[];

const reviewEntry = z.object({ apply: z.boolean(), value: z.string().trim().max(500) });

/**
 * Formulaire de vérification : pour chaque champ, la valeur (modifiable) et le choix de
 * l'appliquer au formulaire. Un champ appliqué doit rester valide pour le formulaire
 * entreprise — nom non vide, site en HTTP(S), taille connue.
 */
export const companyReviewSchema = z
  .object({
    name: reviewEntry,
    sector_id: reviewEntry,
    company_type_id: reviewEntry,
    company_size: reviewEntry,
    website: reviewEntry,
    city: reviewEntry,
    address: reviewEntry,
  })
  .superRefine((review, context) => {
    if (review.name.apply && review.name.value === "") {
      context.addIssue({ code: "custom", path: ["name", "value"], message: "Le nom ne peut pas être vide" });
    }
    if (review.website.apply && review.website.value !== "" && !httpUrl.safeParse(review.website.value).success) {
      context.addIssue({
        code: "custom",
        path: ["website", "value"],
        message: "Le site doit commencer par http:// ou https://",
      });
    }
    if (
      review.company_size.apply &&
      !(COMPANY_SIZES as readonly string[]).includes(review.company_size.value)
    ) {
      context.addIssue({ code: "custom", path: ["company_size", "value"], message: "Taille inconnue" });
    }
  });

export type CompanyReviewInput = z.input<typeof companyReviewSchema>;
export type CompanyReviewValues = z.output<typeof companyReviewSchema>;

/** Valeur actuelle d'un champ du formulaire : vide, ou laissée à sa valeur par défaut. */
function isBlank(field: ResearchedFieldName, current: CompanyFormInput): boolean {
  const value = current[field];
  if (field === "company_size") return value === "UNKNOWN";
  return value === undefined || value === null || value.trim() === "";
}

/**
 * Vérification initiale : chaque valeur trouvée est proposée, mais **appliquée par défaut
 * seulement si le champ du formulaire est vide**. Ce que l'utilisateur a déjà saisi est
 * préservé ; il coche lui-même ce qu'il veut remplacer.
 */
export function initialReview(
  result: CompanyResearchResult,
  current: CompanyFormInput,
): CompanyReviewInput {
  const entry = (field: ResearchedFieldName) => {
    const value = result.fields[field].value ?? "";
    return { apply: value !== "" && isBlank(field, current), value };
  };
  return {
    name: entry("name"),
    sector_id: entry("sector_id"),
    company_type_id: entry("company_type_id"),
    company_size: entry("company_size"),
    website: entry("website"),
    city: entry("city"),
    address: entry("address"),
  };
}

/** Le champ du formulaire a-t-il déjà une valeur que l'application remplacerait ? */
export function replacesInput(field: ResearchedFieldName, current: CompanyFormInput): boolean {
  return !isBlank(field, current);
}

/** Valeurs à verser dans le formulaire entreprise : seulement les champs cochés. */
export function reviewToFormPatch(review: CompanyReviewValues): Partial<CompanyFormInput> {
  const patch: Partial<CompanyFormInput> = {};
  for (const field of RESEARCHED_FIELDS) {
    const entry = review[field];
    if (!entry.apply) continue;
    if (field === "company_size") {
      const size = COMPANY_SIZES.find((value) => value === entry.value);
      if (size) patch.company_size = size;
    } else {
      patch[field] = entry.value;
    }
  }
  return patch;
}
