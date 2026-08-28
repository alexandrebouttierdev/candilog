import { z } from "zod";

/**
 * Champ texte facultatif : normalise « vide » en `null`.
 *
 * Un `input` non renseigné vaut `""`, jamais `undefined`. Sans cette normalisation, le
 * backend recevrait des chaînes vides là où il attend `Option<String>`, et la base
 * stockerait `''` au lieu de `NULL` — deux valeurs que `coalesce` et les `LIKE` des
 * requêtes de recherche ne traitent pas de la même façon.
 */
export const textFacultatif = z
  .string()
  .trim()
  .transform((value) => (value === "" ? null : value))
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
    .transform((value) => (value === "" ? null : value))
    .nullable()
    .default(null)
    .refine(
      (value) => {
        if (value === null) return true;
        try {
          return ["http:", "https:"].includes(new URL(value).protocol);
        } catch {
          return false;
        }
      },
      { message },
    );

/** Id facultatif venant d'un sélecteur : `""` signifie « aucun ». */
export const idFacultatif = z
  .string()
  .transform((value) => (value === "" ? null : value))
  .nullable()
  .default(null);
