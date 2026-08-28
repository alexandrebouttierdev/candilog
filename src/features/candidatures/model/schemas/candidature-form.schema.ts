import { z } from "zod";
import { texteFacultatif, urlFacultative } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";

/**
 * Formulaire candidature, création et modification.
 *
 * Reprend les règles de `CandidatureService::valider` côté Rust : poste requis, date
 * valide, lien limité à HTTP(S). L'entreprise est requise ici **et** en base par une clé
 * étrangère `NOT NULL` : une candidature sans entreprise n'a pas de sens métier.
 */
export const candidatureFormSchema = z.object({
  poste: z.string().trim().min(1, "Le poste est obligatoire"),
  entrepriseId: z.string().min(1, "L'entreprise est obligatoire"),
  typeContrat: z.enum(["CDI", "CDD", "Freelance", "Stage", "Alternance", "Interim", "Autre"]),
  statut: z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"]),
  dateEnvoi: z
    .string()
    .trim()
    .min(1, "La date d'envoi est obligatoire")
    .refine((valeur) => versDateIso(valeur) !== null, {
      message: `Date invalide — format attendu ${FORMAT_DATE}.`,
    })
    // Transformée dès la validation : le ViewModel et le backend ne manipulent que de l'ISO.
    .transform((valeur) => versDateIso(valeur) as string),
  lienOffre: urlFacultative("Le lien de l'offre doit commencer par http:// ou https://"),
  notes: texteFacultatif,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type CandidatureFormValues = z.output<typeof candidatureFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type CandidatureFormInput = z.input<typeof candidatureFormSchema>;
