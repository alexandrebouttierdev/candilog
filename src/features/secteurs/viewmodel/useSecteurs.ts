import { useQuery } from "@tanstack/react-query";
import { secteurService } from "../services/secteur.service";

/** Clé de cache du référentiel. */
export const SECTEURS_KEY = ["secteurs"] as const;

/**
 * Charge le référentiel des secteurs pour les sélecteurs de formulaire.
 *
 * `staleTime: Infinity` : le référentiel est figé au démarrage du backend et ne change pas
 * pendant la session. Le recharger à chaque ouverture de formulaire serait un aller-retour
 * IPC pour un résultat identique.
 */
export function useSecteurs() {
  return useQuery({
    queryKey: SECTEURS_KEY,
    queryFn: secteurService.lister,
    staleTime: Number.POSITIVE_INFINITY,
  });
}
