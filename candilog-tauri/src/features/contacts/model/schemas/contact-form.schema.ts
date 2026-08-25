import { z } from "zod";
import {
  identifiantFacultatif,
  texteFacultatif,
  urlFacultative,
} from "@/shared/lib/zod-helpers";

/**
 * Formulaire contact, création et modification.
 *
 * Reprend les règles de `ContactService::valider` côté Rust : prénom et nom requis, profil
 * LinkedIn limité à HTTP(S). L'e-mail est validé ici seulement — le backend ne l'impose pas,
 * et refuser à l'enregistrement une adresse recopiée d'une carte de visite serait plus
 * gênant qu'utile.
 */
export const contactFormSchema = z.object({
  prenom: z.string().trim().min(1, "Le prénom est obligatoire"),
  nom: z.string().trim().min(1, "Le nom est obligatoire"),
  email: z
    .string()
    .trim()
    .transform((valeur) => (valeur === "" ? null : valeur))
    .nullable()
    .default(null)
    .refine((valeur) => valeur === null || z.email().safeParse(valeur).success, {
      message: "Adresse e-mail invalide",
    }),
  telephone: texteFacultatif,
  entrepriseId: identifiantFacultatif,
  poste: texteFacultatif,
  roleSuivi: texteFacultatif,
  linkedin: urlFacultative("Le profil LinkedIn doit commencer par http:// ou https://"),
  notes: texteFacultatif,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type ContactFormValues = z.output<typeof contactFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type ContactFormInput = z.input<typeof contactFormSchema>;
