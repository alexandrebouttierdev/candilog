import { z } from "zod";
import { idFacultatif, textFacultatif, urlFacultative } from "@/shared/lib/zod-helpers";

/**
 * Formulaire entreprise, création et modification.
 *
 * Un seul schéma pour les deux : les règles sont identiques. Elles reprennent celles de
 * `CompanyService::valider` côté Rust — la validation frontend sert l'ergonomie, le backend
 * reste seul garant.
 *
 * Type et taille sont deux champs distincts : une ESN peut être une PME, un éditeur SaaS
 * une grande entreprise. Les fondre en un seul rendrait la moitié des combinaisons
 * inexprimables.
 */
export const companyFormSchema = z.object({
  name: z.string().trim().min(1, "Le nom de l'entreprise est obligatoire"),
  sector_id: idFacultatif,
  company_type_id: idFacultatif,
  company_size: z.enum(["MICRO", "TPE", "PME", "ETI", "LARGE", "UNKNOWN"]),
  website: urlFacultative("Le site web doit commencer par http:// ou https://"),
  city: textFacultatif,
  address: textFacultatif,
  notes: textFacultatif,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type CompanyFormValues = z.output<typeof companyFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type CompanyFormInput = z.input<typeof companyFormSchema>;
