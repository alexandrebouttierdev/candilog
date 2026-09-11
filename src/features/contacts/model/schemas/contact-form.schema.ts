import { z } from "zod";
import {
  optionalId,
  optionalText,
  optionalUrl,
} from "@/shared/lib/zod-helpers";

/**
 * Form contact, création et modification.
 *
 * Reprend les règles de `ContactService::valider` côté Rust : prénom et nom requis, profil
 * LinkedIn limité à HTTP(S). L'e-mail est validé ici seulement — le backend ne l'impose pas,
 * et refuser à l'enregistrement une adresse recopiée d'une carte de visite serait plus
 * gênant qu'utile.
 */
export const contactFormSchema = z.object({
  first_name: z.string().trim().min(1, "Le prénom est obligatoire"),
  name: z.string().trim().min(1, "Le nom est obligatoire"),
  email: z
    .string()
    .trim()
    .transform((value) => (value === "" ? null : value))
    .nullable()
    .default(null)
    .refine((value) => value === null || z.email().safeParse(value).success, {
      message: "Adresse e-mail invalide",
    }),
  phone: optionalText,
  company_id: optionalId,
  job_title: optionalText,
  tracking_role: optionalText,
  linkedin: optionalUrl("Le profil LinkedIn doit commencer par http:// ou https://"),
  notes: optionalText,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type ContactFormValues = z.output<typeof contactFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type ContactFormInput = z.input<typeof contactFormSchema>;
