import { ipc } from "@/shared/services/ipc";
import type {
  About,
  LlmForm,
  UpdateInfo,
  Settings,
} from "@/shared/types/generated/settings";

export type * from "@/shared/types/generated/settings";

/** Frontière IPC des réglages, sauvegardes et mises à jour. */
export const settingsService = {
  load: () => ipc<Settings>("settings_load"),
  save: (settings: Settings) =>
    ipc<Settings>("settings_save", { settings }),
  testConnection: (llm: LlmForm) =>
    ipc<void>("settings_test_connection", { llm }),
  listModels: (llm: LlmForm) =>
    ipc<string[]>("settings_list_models", { llm }),
  clearAiCache: () => ipc<void>("settings_clear_ai_cache"),
  export: (path: string) => ipc<void>("settings_export", { path }),
  restore: (path: string) => ipc<void>("settings_restore", { path }),
  reset: () => ipc<void>("settings_reset"),
  checkUpdate: () => ipc<UpdateInfo | null>("settings_check_update"),
  downloadUpdate: (url: string, name: string) =>
    ipc<string>("settings_download_update", { url, name }),
  about: () => ipc<About>("settings_about"),
};
