import { z } from "zod";

/**
 * Champ texte facultatif : normalise « vide » en `null`.
 *
 * Un `input` non renseigné vaut `""`, jamais `undefined`. Sans cette normalisation, le
 * backend recevrait des chaînes vides là où il attend `Option<String>`, et la base
 * stockerait `''` au lieu de `NULL` — deux valeurs que `coalesce` et les `LIKE` des
 * requêtes de recherche ne traitent pas de la même façon.
 */
export const texteFacultatif = z
  .string()
  .trim()
  .transform((valeur) => (valeur === "" ? null : valeur))
  .nullable()
  .default(null);

/**
 * URL facultative, limitée à HTTP(S).
 *
 * Reprend la règle appliquée par le backend (`core::utils::validation`) : un champ saisi
 * librement puis ouvert d'un clic ne doit pas pouvoir porter un `javascript:`.
 */
export const urlFacultative = (message = "Adresse invalide — attendu http:// ou https://") =>
  z
    .string()
    .trim()
    .transform((valeur) => (valeur === "" ? null : valeur))
    .nullable()
    .default(null)
    .refine(
      (valeur) => {
        if (valeur === null) return true;
        try {
          return ["http:", "https:"].includes(new URL(valeur).protocol);
        } catch {
          return false;
        }
      },
      { message },
    );

/** Identifiant facultatif venant d'un sélecteur : `""` signifie « aucun ». */
export const identifiantFacultatif = z
  .string()
  .transform((valeur) => (valeur === "" ? null : valeur))
  .nullable()
  .default(null);
