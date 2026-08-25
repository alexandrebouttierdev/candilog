import { QueryClient } from "@tanstack/react-query";
import { AppError } from "@/shared/types/app-error";

/**
 * Client TanStack Query de l'application.
 *
 * Les données proviennent d'une base SQLite locale : aucune latence réseau, aucune
 * concurrence entre plusieurs clients. Les réglages par défaut de TanStack Query, pensés
 * pour une API distante, provoqueraient ici des rechargements inutiles — d'où
 * `refetchOnWindowFocus` désactivé et un `staleTime` non nul : rien ne change dans le dos
 * de l'application, les mutations invalidant elles-mêmes ce qu'elles touchent.
 */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: false,
        staleTime: 30_000,
        retry: (failureCount, error) => {
          // Une validation refusée ou une ressource absente ne deviendra pas vraie en
          // réessayant ; seules les erreurs réseau des fournisseurs IA le méritent.
          if (error instanceof AppError) {
            return error.code === "HTTP_ERROR" && failureCount < 2;
          }
          return false;
        },
      },
      mutations: { retry: false },
    },
  });
}
