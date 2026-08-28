import { z } from "zod";
import { textFacultatif, urlFacultative } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";

/**
 * Form candidature, création et modification.
 *
 * Reprend les règles de `ApplicationService::valider` côté Rust : poste requis, date
 * valide, lien limité à HTTP(S). L'entreprise est requise ici **et** en base par une clé
 * étrangère `NOT NULL` : une candidature sans entreprise n'a pas de sens métier.
 */
export const applicationFormSchema = z.object({
  job_title: z.string().trim().min(1, "Le poste est obligatoire"),
  company_id: z.string().min(1, "L'entreprise est obligatoire"),
  contract_type: z.enum(["CDI", "CDD", "Freelance", "Stage", "Alternance", "Interim", "Autre"]),
  status: z.enum(["EN_ATTENTE", "RELANCEE", "ENTRETIEN", "REFUS"]),
  sent_date: z
    .string()
    .trim()
    .min(1, "La date d'envoi est obligatoire")
    .refine((value) => versDateIso(value) !== null, {
      message: `Date invalide — format attendu ${FORMAT_DATE}.`,
    })
    // Transformée dès la validation : le ViewModel et le backend ne manipulent que de l'ISO.
    .transform((value) => versDateIso(value) as string),
  job_url: urlFacultative("Le lien de l'offre doit commencer par http:// ou https://"),
  notes: textFacultatif,
});

/** Values validées, telles qu'envoyées au backend. */
export type ApplicationFormValues = z.output<typeof applicationFormSchema>;

/** Values saisies, avant transformation — ce que manipule React Hook Form. */
export type ApplicationFormInput = z.input<typeof applicationFormSchema>;
