import { z } from "zod";
import { textFacultatif } from "@/shared/lib/zod-helpers";
import { FORMAT_DATE, versDateIso } from "@/shared/lib/dates";

/**
 * Form relance, création et modification.
 *
 * Contrairement à l'entretien, la relance se programme au jour : pas de champ heure, et la
 * date part en `AAAA-MM-JJ`, format que les requêtes de plage du calendrier savent borner.
 */
export const followUpFormSchema = z.object({
  application_id: z.string().min(1, "La candidature concernée est obligatoire"),
  follow_up_date: z
    .string()
    .trim()
    .min(1, "La date est obligatoire")
    .refine((value) => versDateIso(value) !== null, {
      message: `Date invalide — format attendu ${FORMAT_DATE}.`,
    })
    .transform((value) => versDateIso(value) as string),
  type: z.string().trim().min(1, "Le canal est obligatoire"),
  notes: textFacultatif,
});

/** Values validées, telles qu'envoyées au backend. */
export type FollowUpFormValues = z.output<typeof followUpFormSchema>;

/** Values saisies, avant transformation — ce que manipule React Hook Form. */
export type FollowUpFormInput = z.input<typeof followUpFormSchema>;
