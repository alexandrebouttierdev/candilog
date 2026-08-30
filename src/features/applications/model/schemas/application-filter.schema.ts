import { z } from "zod";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";
import { MAX_WEEKLY_HOURS } from "./application-form.schema";

/**
 * Borne de période facultative : vide vaut « pas de borne », sinon la date doit exister.
 *
 * `superRefine` et non `refine` après transformation : il faut distinguer « champ vide »
 * de « date que `versDateIso` a refusée », que la valeur transformée réduit toutes deux à
 * `null`.
 */
function borneFacultative() {
  return z
    .string()
    .trim()
    .default("")
    .superRefine((value, ctx) => {
      if (value !== "" && versDateIso(value) === null) {
        ctx.addIssue({
          code: "custom",
          message: `Date invalide — format attendu ${FORMAT_DATE}.`,
        });
      }
    })
    .transform((value) => (value === "" ? null : versDateIso(value)));
}

/** Borne d'heures hebdomadaires facultative, saisie au clavier. */
function heuresFacultatives() {
  return z
    .string()
    .trim()
    .default("")
    .superRefine((value, ctx) => {
      if (value === "") return;
      const hours = Number(value.replace(",", "."));
      if (!Number.isFinite(hours) || hours <= 0 || hours > MAX_WEEKLY_HOURS) {
        ctx.addIssue({
          code: "custom",
          message: `Indiquez un nombre d'heures entre 0 et ${MAX_WEEKLY_HOURS}.`,
        });
      }
    })
    .transform((value) => (value === "" ? null : Number(value.replace(",", "."))));
}

/**
 * Filtres cumulables de la vue Liste et du Kanban.
 *
 * Schéma distinct de celui du formulaire : les règles n'ont rien à voir. Un filtre vide est
 * légitime — c'est l'état par défaut de l'écran — alors qu'un formulaire vide ne l'est pas.
 *
 * Les codes des référentiels ne sont pas énumérés ici : leur jeu de valeurs vit en base, et
 * le recopier dans un `z.enum` en ferait une seconde source de vérité à maintenir. Les
 * options proposées viennent du référentiel chargé, et le backend refuse ce qu'il ne
 * connaît pas.
 */
export const applicationFilterSchema = z
  .object({
    status: z.array(z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"])).default([]),
    application_type: z.array(z.enum(["OFFRE", "SPONTANEE"])).default([]),
    contract_type_code: z.array(z.string()).default([]),
    professional_domain_id: z.array(z.string()).default([]),
    company_type_id: z.array(z.string()).default([]),
    company_size: z
      .array(z.enum(["MICRO", "TPE", "PME", "ETI", "LARGE", "UNKNOWN"]))
      .default([]),
    sector_id: z.array(z.string()).default([]),
    weekly_work_schedule: z
      .array(z.enum(["FULL_TIME", "PART_TIME", "UNSPECIFIED"]))
      .default([]),
    min_weekly_hours: heuresFacultatives(),
    max_weekly_hours: heuresFacultatives(),
    company_id: z
      .string()
      .transform((value) => (value === "" ? null : value))
      .nullable()
      .default(null),
    city: z.string().trim().default(""),
    job_title: z.string().trim().default(""),
    start_date: borneFacultative(),
    end_date: borneFacultative(),
  })
  .superRefine((filter, ctx) => {
    // Une période inversée ne renvoie jamais rien : sans ce contrôle, l'écran afficherait
    // un état vide indiscernable d'une absence réelle de candidatures.
    if (filter.start_date && filter.end_date && filter.start_date > filter.end_date) {
      ctx.addIssue({
        code: "custom",
        path: ["end_date"],
        message: "La fin de période précède son début.",
      });
    }
    // Même raisonnement pour l'amplitude horaire.
    if (
      filter.min_weekly_hours !== null &&
      filter.max_weekly_hours !== null &&
      filter.min_weekly_hours > filter.max_weekly_hours
    ) {
      ctx.addIssue({
        code: "custom",
        path: ["max_weekly_hours"],
        message: "Le maximum d'heures est inférieur au minimum.",
      });
    }
  });

export type ApplicationFilterValues = z.output<typeof applicationFilterSchema>;
export type ApplicationFilterInput = z.input<typeof applicationFilterSchema>;

/** Filtre vide, état par défaut de l'écran. */
export const FILTER_VIDE: ApplicationFilterValues = {
  status: [],
  application_type: [],
  contract_type_code: [],
  professional_domain_id: [],
  company_type_id: [],
  company_size: [],
  sector_id: [],
  weekly_work_schedule: [],
  min_weekly_hours: null,
  max_weekly_hours: null,
  company_id: null,
  city: "",
  job_title: "",
  start_date: null,
  end_date: null,
};
