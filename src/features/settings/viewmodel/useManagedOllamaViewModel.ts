import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  ManagedModelId,
  ManagedOllamaDownloadProgress,
  ManagedRuntimeState,
} from "@/shared/types/generated/ai";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import {
  managedOllamaService,
  MANAGED_OLLAMA_KEY,
} from "../services/managedOllamaService";
import { SETTINGS_KEY } from "./useSettingsViewModel";

function errorMessage(error: unknown): string {
  return error instanceof AppError ? error.message : "L'opération Ollama géré a échoué.";
}

export function useManagedOllamaViewModel(
  onConfigured?: () => void,
  options: { enabled?: boolean } = {},
) {
  const enabled = options.enabled ?? true;
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [progress, setProgress] = useState<ManagedOllamaDownloadProgress | null>(null);
  const [transientState, setTransientState] = useState<ManagedRuntimeState | null>(null);
  const [eventError, setEventError] = useState<string | null>(null);

  const query = useQuery({
    queryKey: MANAGED_OLLAMA_KEY,
    queryFn: managedOllamaService.status,
    enabled,
  });

  useEffect(() => {
    if (!enabled) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void managedOllamaService.onProgress((event) => {
      setProgress(event);
      setTransientState(event.state);
      setEventError(null);
      if (event.state === "ready" && event.progress >= 100) {
        setProgress(null);
        void queryClient.invalidateQueries({ queryKey: MANAGED_OLLAMA_KEY });
        void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      }
    }).then((fn) => {
      if (disposed) fn();
      else unlisten = fn;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [enabled, queryClient]);

  const runtimeState = useMemo<ManagedRuntimeState>(() => {
    if (!enabled) return "not_installed";
    if (transientState) return transientState;
    if (query.isPending) return "starting";
    return query.data?.runtime_state ?? "not_installed";
  }, [enabled, query.data?.runtime_state, query.isPending, transientState]);

  const install = useMutation({
    mutationFn: async (modelId: ManagedModelId) => {
      setTransientState("downloading");
      setEventError(null);
      return managedOllamaService.install({ model_id: modelId });
    },
    onSuccess: (status) => {
      queryClient.setQueryData(MANAGED_OLLAMA_KEY, status);
      setProgress(null);
      setTransientState(status.runtime_state);
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      onConfigured?.();
      notify({ tone: "success", title: "Modèle local prêt" });
    },
    onError: (error: unknown) => {
      if (error instanceof AppError && error.isCancelled) {
        setProgress(null);
        setEventError(null);
        setTransientState(null);
        void queryClient.invalidateQueries({ queryKey: MANAGED_OLLAMA_KEY });
        notify({ tone: "info", title: "Téléchargement annulé" });
        return;
      }
      setTransientState("error");
      setEventError(errorMessage(error));
    },
  });

  const remove = useMutation({
    mutationFn: managedOllamaService.remove,
    onSuccess: (status) => {
      queryClient.setQueryData(MANAGED_OLLAMA_KEY, status);
      setProgress(null);
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      notify({ tone: "success", title: "Modèle local supprimé" });
    },
    onError: (error: unknown) => {
      notify({ tone: "error", title: "Suppression impossible", detail: errorMessage(error) });
    },
  });

  const activate = useMutation({
    mutationFn: managedOllamaService.activate,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: MANAGED_OLLAMA_KEY });
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      onConfigured?.();
      notify({ tone: "success", title: "Modèle activé" });
    },
    onError: (error: unknown) => {
      notify({ tone: "error", title: "Activation impossible", detail: errorMessage(error) });
    },
  });

  return {
    runtimeState,
    status: query.data ?? null,
    progress,
    error: eventError ?? (query.error ? errorMessage(query.error) : null),
    isInstalling: install.isPending,
    isRemoving: remove.isPending,
    isActivating: activate.isPending,
    install: install.mutate,
    cancel: () => void managedOllamaService.cancel(),
    remove: remove.mutate,
    activate: activate.mutate,
    recharger: () => void query.refetch(),
  };
}

export type ManagedOllamaViewModel = ReturnType<typeof useManagedOllamaViewModel>;
