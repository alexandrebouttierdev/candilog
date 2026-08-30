import { ipc } from "@/shared/services/ipc";
import type {
  About,
  LlmForm,
  ResetOutcome,
  UpdateInfo,
  Settings,
} from "@/shared/types/generated/settings";

export type * from "@/shared/types/generated/settings";

/** Frontière IPC des réglages, sauvegardes et mises à jour. */
export const settingsService = {
  load: () => ipc<Settings>("settings_load"),
  save: (settings: Settings, api_key?: string | null) =>
    ipc<Settings>("settings_save", { settings, api_key: api_key || null }),
  clearApiKey: () => ipc<void>("settings_clear_api_key"),
  testConnection: (llm: LlmForm, api_key?: string | null) =>
    ipc<void>("settings_test_connection", { llm, api_key: api_key || null }),
  listModels: (llm: LlmForm, api_key?: string | null) =>
    ipc<string[]>("settings_list_models", { llm, api_key: api_key || null }),
  clearAiCache: () => ipc<void>("settings_clear_ai_cache"),
  export: () => ipc<boolean>("settings_export"),
  restore: () => ipc<boolean>("settings_restore"),
  reset: () => ipc<ResetOutcome>("settings_reset"),
  checkUpdate: () => ipc<UpdateInfo | null>("settings_check_update"),
  downloadUpdate: (url: string, name: string) =>
    ipc<string>("settings_download_update", { url, name }),
  about: () => ipc<About>("settings_about"),
};
