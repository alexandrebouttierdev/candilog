import { z } from "zod";
import { texteFacultatif } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";

/**
 * Formulaire relance, création et modification.
 *
 * Contrairement à l'entretien, la relance se programme au jour : pas de champ heure, et la
 * date part en `AAAA-MM-JJ`, format que les requêtes de plage du calendrier savent borner.
 */
export const relanceFormSchema = z.object({
  candidatureId: z.string().min(1, "La candidature concernée est obligatoire"),
  dateRelance: z
    .string()
    .trim()
    .min(1, "La date est obligatoire")
    .refine((valeur) => versDateIso(valeur) !== null, {
      message: `Date invalide — format attendu ${FORMAT_DATE}.`,
    })
    .transform((valeur) => versDateIso(valeur) as string),
  type: z.string().trim().min(1, "Le canal est obligatoire"),
  notes: texteFacultatif,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type RelanceFormValues = z.output<typeof relanceFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type RelanceFormInput = z.input<typeof relanceFormSchema>;
