import { useQuery } from "@tanstack/react-query";
import { settingsService } from "../services/settingsService";
import { A_ABOUT_KEY } from "./useSettingsViewModel";

/** Identité produit (version, nom) pour l'écran À propos. */
export function useAboutViewModel() {
  const query = useQuery({ queryKey: A_ABOUT_KEY, queryFn: settingsService.about });

  return {
    version: query.data?.version ?? "…",
    name: query.data?.name,
    isLoading: query.isPending,
    error: query.error,
    recharger: () => void query.refetch(),
  };
}
