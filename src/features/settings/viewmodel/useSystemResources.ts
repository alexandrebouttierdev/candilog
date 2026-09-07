import { useQuery } from "@tanstack/react-query";
import {
  SYSTEM_RESOURCES_KEY,
  systemResourceService,
} from "../services/systemResourceService";

/** Poll léger des ressources machine pour le rail (CPU / RAM / VRAM). */
export function useSystemResources() {
  return useQuery({
    queryKey: SYSTEM_RESOURCES_KEY,
    queryFn: systemResourceService.snapshot,
    refetchInterval: 2_500,
    refetchIntervalInBackground: false,
    staleTime: 2_000,
  });
}
