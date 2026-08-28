import { z } from "zod";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";

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

/**
 * Filters cumulables de la vue List et du Kanban.
 *
 * Schéma distinct de celui du formulaire : les règles n'ont rien à voir. Un filtre vide est
 * légitime — c'est l'état par défaut de l'écran — alors qu'un formulaire vide ne l'est pas.
 */
export const applicationFilterSchema = z
  .object({
    status: z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"]).nullable().default(null),
    contract: z
      .enum(["CDI", "CDD", "Freelance", "Stage", "Alternance", "Interim", "Autre"])
      .nullable()
      .default(null),
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
  });

export type ApplicationFilterValues = z.output<typeof applicationFilterSchema>;
export type ApplicationFilterInput = z.input<typeof applicationFilterSchema>;

/** Filter vide, état par défaut de l'écran. */
export const FILTER_VIDE: ApplicationFilterInput = {
  status: null,
  contract: null,
  company_id: null,
  city: "",
  job_title: "",
  start_date: "",
  end_date: "",
};
