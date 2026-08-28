import { z } from "zod";
import {
  idFacultatif,
  textFacultatif,
  urlFacultative,
} from "@/shared/lib/zod-helpers";

/**
 * Form entreprise, création et modification.
 *
 * Un seul schéma pour les deux : les règles sont identiques (MIGRATION.md §12). Elles
 * reprennent celles de `CompanyService::valider` côté Rust — la validation frontend sert
 * l'ergonomie, le backend reste seul garant (§14).
 */
export const companyFormSchema = z.object({
  name: z.string().trim().min(1, "Le nom de l'entreprise est obligatoire"),
  sector_id: idFacultatif,
  type: textFacultatif,
  website: urlFacultative("Le site web doit commencer par http:// ou https://"),
  city: textFacultatif,
  address: textFacultatif,
  notes: textFacultatif,
});

/** Values validées, telles qu'envoyées au backend. */
export type CompanyFormValues = z.output<typeof companyFormSchema>;

/** Values saisies, avant transformation — ce que manipule React Hook Form. */
export type CompanyFormInput = z.input<typeof companyFormSchema>;
