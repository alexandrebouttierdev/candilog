import { useQuery } from "@tanstack/react-query";
import { analyticsService } from "../services/analyticsService";
import { ANALYTICS_KEY } from "./analyticsKeys";
import { localIso } from "../model/agenda";

/**
 * Échéances des sept prochains jours (relances non faites, retards compris, et entretiens).
 * Partagée par l'écran Aujourd'hui et le décompte de la navigation : une seule requête.
 */
export function useAgenda() {
  const today = localIso();
  const query = useQuery({ queryKey: [...ANALYTICS_KEY, "agenda", today], queryFn: analyticsService.agenda });
  return { today, ...query };
}
