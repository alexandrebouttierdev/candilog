import { useQuery } from "@tanstack/react-query";
import { applicationService } from "../services/applicationService";
import { APPLICATIONS_KEY } from "./useApplicationsViewModel";

/**
 * Historique des statuts d'une candidature, pour l'inspecteur.
 *
 * Rangé sous la clé racine de la feature : toute écriture sur les candidatures
 * l'invalide, donc un changement de statut apparaît aussitôt dans l'historique.
 */
export function useStatusHistory(id: string | null) {
  return useQuery({
    queryKey: [...APPLICATIONS_KEY, "historique", id],
    queryFn: () => applicationService.statusHistory(id as string),
    enabled: id !== null,
  });
}

/** Ce qu'emporterait la suppression, chargé à l'ouverture du dialogue. */
export function useDeletionImpact(id: string | null) {
  return useQuery({
    queryKey: [...APPLICATIONS_KEY, "impact", id],
    queryFn: () => applicationService.deletionImpact(id as string),
    enabled: id !== null,
    // Toujours recompté à l'ouverture : une relance ajoutée depuis changerait le décompte.
    staleTime: 0,
  });
}
