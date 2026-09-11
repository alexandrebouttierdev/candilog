export { isAiConfigured, aiStatus, type AiStatus, type ConnectionTest } from "./model/aiStatus";
export { managedOllamaStatus, isManagedOllamaBusy } from "./model/managedOllamaStatus";
export {
  PROVIDERS,
  OLLAMA_PROVIDER,
  OTHER_PROVIDERS,
  defaultEndpoint,
  defaultModel,
  isCustomProvider,
  getProvider,
  idProvider,
  toProvider,
  presetFromLlm,
  llmFromPreset,
  type ProviderOption,
} from "./model/providers";
export {
  ManagedPublisherLogo,
  ProviderGrid,
  logoManagedPublisher,
  providerLogo,
} from "./view/components/ProviderGrid";
export { settingsService } from "./services/settingsService";
export { managedOllamaService, MANAGED_OLLAMA_KEY } from "./services/managedOllamaService";

export {
  useSettingsViewModel,
  useBootstrapTheme,
  useThemePreference,
  SETTINGS_KEY,
} from "./viewmodel/useSettingsViewModel";
export { useAboutViewModel } from "./viewmodel/useAboutViewModel";
export { useManagedOllamaViewModel } from "./viewmodel/useManagedOllamaViewModel";
export type { ManagedOllamaViewModel } from "./viewmodel/useManagedOllamaViewModel";
