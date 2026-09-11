import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ManagedModelId } from "@/shared/types/generated/ai";
import type { LlmForm, Settings } from "@/shared/types/generated/settings";
import {
  managedOllamaService,
  MANAGED_OLLAMA_KEY,
  SETTINGS_KEY,
  settingsService,
  toProvider,
  type ProviderOption,
} from "@/features/settings";

/** Données et mutations du sélecteur IA rapide — la vue ne parle plus aux services. */
export function useAiQuickSelectorViewModel() {
  const queryClient = useQueryClient();

  const settings = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: settingsService.load,
  });

  const managed = useQuery({
    queryKey: MANAGED_OLLAMA_KEY,
    queryFn: managedOllamaService.status,
  });

  const saveSettings = useMutation({
    mutationFn: (next: Settings) => settingsService.save(next, null),
    onSuccess: (saved) => {
      queryClient.setQueryData(SETTINGS_KEY, saved);
    },
  });

  const activateManaged = useMutation({
    mutationFn: (modelId: ManagedModelId) => managedOllamaService.activate(modelId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: SETTINGS_KEY });
      void queryClient.invalidateQueries({ queryKey: MANAGED_OLLAMA_KEY });
    },
  });

  return {
    settings: settings.data,
    managed: managed.data,
    isSettingsLoading: settings.isPending,
    saveSettings: saveSettings.mutateAsync,
    activateManaged: activateManaged.mutate,
    activateManagedAsync: activateManaged.mutateAsync,
    isSaving: saveSettings.isPending,
    isActivating: activateManaged.isPending,
    listModels: (providerId: ProviderOption["id"], llm: LlmForm) =>
      settingsService.listModels({ ...llm, provider: toProvider(providerId) }, null),
  };
}
