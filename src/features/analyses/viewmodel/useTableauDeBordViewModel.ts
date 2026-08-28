import { useQuery } from "@tanstack/react-query";
import { analysesService } from "../services/analyses.service";

/** Racine commune : une mutation de candidature, entretien ou relance peut l'invalider. */
export const ANALYSES_KEY = ["analyses"] as const;

/** Charge utile unique du tableau de bord, déjà agrégée côté `SQLite`. */
export function useTableauDeBordViewModel() {
  const requete = useQuery({
    queryKey: [...ANALYSES_KEY, "tableau-de-bord"],
    queryFn: analysesService.tableauDeBord,
  });

  return {
    data: requete.data,
    isLoading: requete.isPending,
    error: requete.error,
    recharger: () => void requete.refetch(),
  };
}
