import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import type { SyncStatus } from "@/shared/types/generated/sync";
import { syncService } from "../services/syncService";

export const SYNC_STATUS_KEY = ["sync", "status"] as const;
export const SYNC_CONFLICTS_KEY = ["sync", "conflicts"] as const;
export const SYNC_AVAILABILITY_KEY = ["sync", "availability"] as const;

/** État de la synchronisation, tenu à jour par l'événement `sync://status`. */
export function useSyncStatus() {
  return useQuery({ queryKey: SYNC_STATUS_KEY, queryFn: syncService.status });
}

/** Phases à l'issue desquelles des données ont pu arriver d'un autre ordinateur. */
const WORKING = new Set(["syncing", "restoring"]);

/**
 * Pont unique entre l'événement d'état et le cache : posé une fois par la coque.
 *
 * Quand un cycle se termine, les écrans affichés relisent leurs données — un autre
 * ordinateur a pu en modifier. Hors de Tauri (revue dans un navigateur), l'écoute échoue
 * silencieusement et l'état reste celui de la dernière lecture.
 */
export function useSyncEvents() {
  const queryClient = useQueryClient();

  useEffect(() => {
    let cancelled = false;
    let dispose: (() => void) | undefined;
    void listen<SyncStatus>("sync://status", (event) => {
      if (cancelled) return;
      const previous = queryClient.getQueryData<SyncStatus>(SYNC_STATUS_KEY);
      queryClient.setQueryData(SYNC_STATUS_KEY, event.payload);
      const finished = previous !== undefined && WORKING.has(previous.phase) && !WORKING.has(event.payload.phase);
      if (finished) {
        void queryClient.invalidateQueries({
          predicate: (query) => query.queryKey[0] !== "sync" || query.queryKey[1] === "conflicts",
        });
      } else if (previous?.conflicts !== event.payload.conflicts) {
        void queryClient.invalidateQueries({ queryKey: SYNC_CONFLICTS_KEY });
      }
    })
      .then((unlisten) => {
        if (cancelled) unlisten();
        else dispose = unlisten;
      })
      .catch(() => {
        // La revue dans un navigateur ne dispose pas du runtime Tauri.
      });
    return () => {
      cancelled = true;
      dispose?.();
    };
  }, [queryClient]);
}
