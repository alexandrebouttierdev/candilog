import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type {
  LocalAiDownloadProgress,
  LocalAiState,
  LocalAiStatus,
  LocalModelId,
} from "@/shared/types/generated/ai";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";
import { localAiService } from "../services/localAiService";
import { SETTINGS_KEY } from "./useSettingsViewModel";

export const LOCAL_AI_KEY = ["parametres", "mistral-local"] as const;

function errorMessage(error: unknown): string {
  return error instanceof AppError ? error.message : "L'opération d'IA locale a échoué.";
}

export function useLocalAiViewModel(onConfigured?: () => void) {
  const queryClient = useQueryClient();
  const notify = useUiStore((state) => state.notify);
  const [progress, setProgress] = useState<LocalAiDownloadProgress | null>(null);
  const [transientState, setTransientState] = useState<LocalAiState | null>(null);
  const [eventError, setEventError] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<string | null>(null);

  const query = useQuery({
    queryKey: LOCAL_AI_KEY,
    queryFn: async () => {
      const [recommendation, status] = await Promise.all([
        localAiService.recommendation(),
        localAiService.status(),
      ]);
      return { recommendation, status };
    },
  });

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];
    const register = (promise: Promise<() => void>) => {
      void promise.then((unlisten) => {
        if (disposed) unlisten();
        else unlisteners.push(unlisten);
      });
    };
    register(
      localAiService.onProgress((event) => {
        setProgress(event);
        setTransientState(event.state);
        setEventError(null);
      }),
    );
    register(
      localAiService.onCompleted(() => {
        setProgress(null);
        setTransientState("ready");
        void queryClient.invalidateQueries({ queryKey: LOCAL_AI_KEY });
        void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      }),
    );
    register(
      localAiService.onError((event) => {
        setTransientState("error");
        setEventError(event.message);
      }),
    );
    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [queryClient]);

  const updateStatus = (status: LocalAiStatus) => {
    queryClient.setQueryData(LOCAL_AI_KEY, (current: typeof query.data) =>
      current ? { ...current, status } : current,
    );
    setTransientState(status.state);
  };

  const install = useMutation({
    mutationFn: async (modelId: LocalModelId) => {
      setTransientState("downloading");
      setEventError(null);
      return localAiService.install({ model_id: modelId });
    },
    onSuccess: (status) => {
      updateStatus(status);
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      onConfigured?.();
      notify({ tone: "success", title: "IA locale prête" });
    },
    onError: (error: unknown) => {
      setTransientState("error");
      setEventError(errorMessage(error));
    },
  });
  const remove = useMutation({
    mutationFn: localAiService.remove,
    onSuccess: (status) => {
      updateStatus(status);
      setProgress(null);
      setTestResult(null);
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      notify({ tone: "success", title: "Modèle local supprimé" });
    },
  });
  const benchmark = useMutation({
    mutationFn: localAiService.benchmark,
    onMutate: () => setTransientState("benchmarking"),
    onSuccess: () => {
      setTransientState("ready");
      void queryClient.invalidateQueries({ queryKey: LOCAL_AI_KEY });
    },
    onError: (error: unknown) => {
      setTransientState("error");
      setEventError(errorMessage(error));
    },
  });
  const test = useMutation({
    mutationFn: localAiService.test,
    onSuccess: setTestResult,
  });

  const state = useMemo<LocalAiState>(() => {
    if (transientState) return transientState;
    if (query.isPending) return "detecting_hardware";
    return query.data?.status.state ?? "not_configured";
  }, [query.data?.status.state, query.isPending, transientState]);

  return {
    state,
    recommendation: query.data?.recommendation ?? null,
    status: query.data?.status ?? null,
    progress,
    error: eventError ?? (query.error ? errorMessage(query.error) : null),
    testResult,
    isRemoving: remove.isPending,
    isTesting: test.isPending,
    install: install.mutate,
    cancel: () => void localAiService.cancel(),
    remove: remove.mutate,
    benchmark: benchmark.mutate,
    test: test.mutate,
    reevaluate: () => {
      setTransientState("detecting_hardware");
      setEventError(null);
      void query.refetch().finally(() => setTransientState(null));
    },
  };
}
