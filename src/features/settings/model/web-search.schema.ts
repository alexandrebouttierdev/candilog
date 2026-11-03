import { optionalUrl } from "@/shared/lib/zod-helpers";

/**
 * Adresse d'une instance SearXNG : vide (instances publiques) ou HTTP(S). Reprend la règle
 * du service Rust des réglages, seul garant.
 */
export const searxngUrlSchema = optionalUrl("L'adresse doit commencer par http:// ou https://").refine(
  (value) => value === null || value.length <= 200,
  { message: "Adresse trop longue" },
);
