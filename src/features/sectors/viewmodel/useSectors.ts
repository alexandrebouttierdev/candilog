import { useQuery } from "@tanstack/react-query";
import { sectorService } from "../services/sectorService";

/** Clé de cache du référentiel. */
export const SECTORS_KEY = ["secteurs"] as const;

/**
 * Payload le référentiel des secteurs pour les sélecteurs de formulaire.
 *
 * `staleTime: Infinity` : le référentiel est figé au démarrage du backend et ne change pas
 * pendant la session. Le recharger à chaque ouverture de formulaire serait un aller-retour
 * IPC pour un résultat identique.
 */
export function useSectors() {
  return useQuery({
    queryKey: SECTORS_KEY,
    queryFn: sectorService.list,
    staleTime: Number.POSITIVE_INFINITY,
  });
}
