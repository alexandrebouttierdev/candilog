import { useQuery } from "@tanstack/react-query";
import { analyticsService } from "../services/analyticsService";

/** Root commune : une mutation de candidature, entretien ou relance peut l'invalider. */
export const ANALYTICS_KEY = ["analyses"] as const;

/** Payload utile unique du tableau de bord, déjà agrégée côté `SQLite`. */
export function useDashboardViewModel() {
  const query = useQuery({
    queryKey: [...ANALYTICS_KEY, "tableau-de-bord"],
    queryFn: analyticsService.dashboard,
  });

  return {
    data: query.data,
    isLoading: query.isPending,
    error: query.error,
    recharger: () => void query.refetch(),
  };
}
