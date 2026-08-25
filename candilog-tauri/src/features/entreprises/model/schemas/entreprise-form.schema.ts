import { z } from "zod";
import {
  identifiantFacultatif,
  texteFacultatif,
  urlFacultative,
} from "@/shared/lib/zod-helpers";

/**
 * Formulaire entreprise, création et modification.
 *
 * Un seul schéma pour les deux : les règles sont identiques (MIGRATION.md §12). Elles
 * reprennent celles de `EntrepriseService::valider` côté Rust — la validation frontend sert
 * l'ergonomie, le backend reste seul garant (§14).
 */
export const entrepriseFormSchema = z.object({
  nom: z.string().trim().min(1, "Le nom de l'entreprise est obligatoire"),
  secteurId: identifiantFacultatif,
  type: texteFacultatif,
  siteWeb: urlFacultative("Le site web doit commencer par http:// ou https://"),
  ville: texteFacultatif,
  adresse: texteFacultatif,
  notes: texteFacultatif,
});

/** Valeurs validées, telles qu'envoyées au backend. */
export type EntrepriseFormValues = z.output<typeof entrepriseFormSchema>;

/** Valeurs saisies, avant transformation — ce que manipule React Hook Form. */
export type EntrepriseFormInput = z.input<typeof entrepriseFormSchema>;
