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
export { AppearanceSettings } from "./view/pages/AppearanceSettings";
export { SettingsRow, SettingsSection } from "./view/components/SettingsSection";
export { BackupsPage } from "./view/pages/BackupsPage";
export { UpdatesPage } from "./view/pages/UpdatesPage";
export { AboutPage } from "./view/pages/AboutPage";
export { AI_TASKS, assignmentOf, mainLabel } from "./model/taskRouting";
export type { Assignment } from "./model/taskRouting";
