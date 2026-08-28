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
    .superRefine((valeur, ctx) => {
      if (valeur !== "" && versDateIso(valeur) === null) {
        ctx.addIssue({
          code: "custom",
          message: `Date invalide — format attendu ${FORMAT_DATE}.`,
        });
      }
    })
    .transform((valeur) => (valeur === "" ? null : versDateIso(valeur)));
}

/**
 * Filtres cumulables de la vue Liste et du Kanban.
 *
 * Schéma distinct de celui du formulaire : les règles n'ont rien à voir. Un filtre vide est
 * légitime — c'est l'état par défaut de l'écran — alors qu'un formulaire vide ne l'est pas.
 */
export const candidatureFilterSchema = z
  .object({
    statut: z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"]).nullable().default(null),
    contrat: z
      .enum(["CDI", "CDD", "Freelance", "Stage", "Alternance", "Interim", "Autre"])
      .nullable()
      .default(null),
    entrepriseId: z
      .string()
      .transform((valeur) => (valeur === "" ? null : valeur))
      .nullable()
      .default(null),
    ville: z.string().trim().default(""),
    poste: z.string().trim().default(""),
    dateDebut: borneFacultative(),
    dateFin: borneFacultative(),
  })
  .superRefine((filtre, ctx) => {
    // Une période inversée ne renvoie jamais rien : sans ce contrôle, l'écran afficherait
    // un état vide indiscernable d'une absence réelle de candidatures.
    if (filtre.dateDebut && filtre.dateFin && filtre.dateDebut > filtre.dateFin) {
      ctx.addIssue({
        code: "custom",
        path: ["dateFin"],
        message: "La fin de période précède son début.",
      });
    }
  });

export type CandidatureFilterValues = z.output<typeof candidatureFilterSchema>;
export type CandidatureFilterInput = z.input<typeof candidatureFilterSchema>;

/** Filtre vide, état par défaut de l'écran. */
export const FILTRE_VIDE: CandidatureFilterInput = {
  statut: null,
  contrat: null,
  entrepriseId: null,
  ville: "",
  poste: "",
  dateDebut: "",
  dateFin: "",
};
